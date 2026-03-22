use std::sync::Arc;

use log::debug;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::lsp_types::request::{GotoDeclarationParams, GotoDeclarationResponse};
use tower_lsp::{Client, LanguageServer, LspService, Server};

use crate::analysis::{
    build_document_symbols, build_folding_ranges, build_goto_symbols, goto_declaration_locations,
    goto_definition_locations, reference_locations, rename_workspace_edit,
};
use crate::constants::{semantic_token_types, LANGUAGE_ID};
use crate::context::AppContext;
use crate::documents::{format_text, on_change, semantic_tokens, TextDocumentChange};
use crate::handlers::{code_action, code_lens, execute_command, hover};
use crate::http_client;

#[derive(Debug)]
struct Backend {
    ctx: Arc<AppContext>,
}

impl Backend {
    fn new(client: Client) -> Self {
        Self {
            ctx: Arc::new(AppContext::new(client)),
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: env!("CARGO_PKG_NAME").to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            offset_encoding: None,
            capabilities: ServerCapabilities {
                document_formatting_provider: Some(OneOf::Left(true)),
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::FULL),
                        save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                            include_text: Some(true),
                        })),
                        ..Default::default()
                    },
                )),
                completion_provider: None,
                workspace: None,
                document_symbol_provider: Some(OneOf::Left(true)),
                definition_provider: Some(OneOf::Left(true)),
                declaration_provider: Some(DeclarationCapability::Simple(true)),
                references_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Left(true)),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),
                code_action_provider: Some(CodeActionProviderCapability::Options(
                    CodeActionOptions {
                        code_action_kinds: Some(vec![
                            CodeActionKind::QUICKFIX,
                            CodeActionKind::REFACTOR,
                        ]),
                        resolve_provider: Some(false),
                        work_done_progress_options: WorkDoneProgressOptions::default(),
                    },
                )),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec![
                        http_client::CMD_START_HTTP_CLIENT.to_string(),
                        http_client::CMD_STOP_HTTP_CLIENT.to_string(),
                    ],
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                }),
                code_lens_provider: Some(CodeLensOptions {
                    resolve_provider: Some(false),
                }),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensRegistrationOptions(
                        SemanticTokensRegistrationOptions {
                            text_document_registration_options: TextDocumentRegistrationOptions {
                                document_selector: Some(vec![DocumentFilter {
                                    language: Some(LANGUAGE_ID.to_string()),
                                    scheme: Some("file".to_string()),
                                    pattern: None,
                                }]),
                            },
                            semantic_tokens_options: SemanticTokensOptions {
                                work_done_progress_options: WorkDoneProgressOptions::default(),
                                legend: SemanticTokensLegend {
                                    token_types: semantic_token_types(),
                                    token_modifiers: vec![],
                                },
                                range: Some(false),
                                full: Some(SemanticTokensFullOptions::Bool(true)),
                            },
                            static_registration_options: StaticRegistrationOptions::default(),
                        },
                    ),
                ),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        debug!("initialized!");
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        on_change(
            &self.ctx,
            TextDocumentChange {
                uri: params.text_document.uri,
                text: &params.text_document.text,
            },
        )
        .await;
        debug!("file opened!");
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        on_change(
            &self.ctx,
            TextDocumentChange {
                text: &params.content_changes[0].text,
                uri: params.text_document.uri,
            },
        )
        .await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        if let Some((_, preview)) = self.ctx.preview_map.remove(&uri) {
            preview.stop();
        }
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let uri = params.text_document.uri.to_string();
        let semantic_tokens = semantic_tokens(&self.ctx, &uri);
        if let Some(tokens) = semantic_tokens {
            return Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
                result_id: None,
                data: tokens,
            })));
        }
        Ok(None)
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        Ok(format_text(&self.ctx, params))
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri.to_string();
        let rope = match self.ctx.document_map.get(&uri) {
            Some(rope) => rope,
            None => return Ok(None),
        };
        let text = rope.to_string();
        let symbols = build_document_symbols(&text, &rope);
        Ok(Some(DocumentSymbolResponse::Nested(symbols)))
    }

    async fn folding_range(&self, params: FoldingRangeParams) -> Result<Option<Vec<FoldingRange>>> {
        let uri = params.text_document.uri.to_string();
        let rope = match self.ctx.document_map.get(&uri) {
            Some(rope) => rope,
            None => return Ok(None),
        };
        let text = rope.to_string();
        let ranges = build_folding_ranges(&text, &rope);
        Ok(Some(ranges))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let rope = match self.ctx.document_map.get(uri.as_str()) {
            Some(rope) => rope,
            None => return Ok(None),
        };
        let text = rope.to_string();
        let symbols = build_goto_symbols(&text, &rope);
        let locations = goto_definition_locations(&symbols, &uri, position);
        if locations.is_empty() {
            Ok(None)
        } else {
            Ok(Some(GotoDefinitionResponse::Array(locations)))
        }
    }

    async fn goto_declaration(
        &self,
        params: GotoDeclarationParams,
    ) -> Result<Option<GotoDeclarationResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let rope = match self.ctx.document_map.get(uri.as_str()) {
            Some(rope) => rope,
            None => return Ok(None),
        };
        let text = rope.to_string();
        let symbols = build_goto_symbols(&text, &rope);
        let locations = goto_declaration_locations(&symbols, &uri, position);
        if locations.is_empty() {
            Ok(None)
        } else {
            Ok(Some(GotoDeclarationResponse::Array(locations)))
        }
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let rope = match self.ctx.document_map.get(uri.as_str()) {
            Some(rope) => rope,
            None => return Ok(None),
        };
        let text = rope.to_string();
        let symbols = build_goto_symbols(&text, &rope);
        let locations = reference_locations(&symbols, &uri, position);
        if locations.is_empty() {
            Ok(None)
        } else {
            Ok(Some(locations))
        }
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let new_name = params.new_name;
        let rope = match self.ctx.document_map.get(uri.as_str()) {
            Some(rope) => rope,
            None => return Ok(None),
        };
        let text = rope.to_string();
        let symbols = build_goto_symbols(&text, &rope);
        Ok(rename_workspace_edit(&symbols, &uri, position, &new_name))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        Ok(hover(&self.ctx, &uri, position))
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = params.text_document.uri;
        let position = params.range.start;
        Ok(code_action(&self.ctx, &uri, position))
    }

    async fn execute_command(
        &self,
        params: ExecuteCommandParams,
    ) -> Result<Option<serde_json::Value>> {
        execute_command(&self.ctx, params).await
    }

    async fn code_lens(&self, params: CodeLensParams) -> Result<Option<Vec<CodeLens>>> {
        let uri = params.text_document.uri;
        Ok(code_lens(&self.ctx, &uri))
    }

    async fn did_change_configuration(&self, _: DidChangeConfigurationParams) {
        debug!("configuration changed!");
    }
}

pub(crate) async fn run() {
    env_logger::init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(Backend::new).finish();
    Server::new(stdin, stdout, socket).serve(service).await;
}
