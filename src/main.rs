use dashmap::DashMap;
use log::{debug, warn};
use ropey::Rope;
use std::env;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::request::{GotoDeclarationParams, GotoDeclarationResponse};
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use tree_sitter::{Node, Parser, Query, QueryCursor, StreamingIterator};
use tree_sitter_highlight::{HighlightConfiguration, HighlightEvent, Highlighter};

#[derive(Debug)]
struct Backend {
    client: Client,
    document_map: DashMap<String, Rope>,
    semantic_tokens_map: DashMap<String, Vec<SemanticToken>>,
    preview_map: DashMap<String, http_client::PreviewHandle>,
}

mod doc;
mod http_client;

const TOKEN_TYPE_NAMESPACE: u32 = 0;
const TOKEN_TYPE_TYPE: u32 = 1;
const TOKEN_TYPE_CLASS: u32 = 2;
const TOKEN_TYPE_ENUM: u32 = 3;
const TOKEN_TYPE_INTERFACE: u32 = 4;
const TOKEN_TYPE_STRUCT: u32 = 5;
const TOKEN_TYPE_TYPE_PARAMETER: u32 = 6;
const TOKEN_TYPE_PARAMETER: u32 = 7;
const TOKEN_TYPE_VARIABLE: u32 = 8;
const TOKEN_TYPE_PROPERTY: u32 = 9;
const TOKEN_TYPE_ENUM_MEMBER: u32 = 10;
const TOKEN_TYPE_EVENT: u32 = 11;
const TOKEN_TYPE_FUNCTION: u32 = 12;
const TOKEN_TYPE_METHOD: u32 = 13;
const TOKEN_TYPE_MACRO: u32 = 14;
const TOKEN_TYPE_KEYWORD: u32 = 15;
const TOKEN_TYPE_MODIFIER: u32 = 16;
const TOKEN_TYPE_COMMENT: u32 = 17;
const TOKEN_TYPE_STRING: u32 = 18;
const TOKEN_TYPE_NUMBER: u32 = 19;
const TOKEN_TYPE_REGEXP: u32 = 20;
const TOKEN_TYPE_OPERATOR: u32 = 21;
const TOKEN_TYPE_DECORATOR: u32 = 22;

const HIGHLIGHT_NAMES: &[&str] = &[
    "comment.documentation",
    "comment.error",
    "comment.warning",
    "comment.todo",
    "comment.note",
    "string.documentation",
    "string.escape",
    "string.regexp",
    "string.special",
    "string.special.symbol",
    "string.special.path",
    "string.special.url",
    "character.special",
    "number.float",
    "type.builtin",
    "type.definition",
    "variable.builtin",
    "variable.parameter",
    "variable.parameter.builtin",
    "variable.member",
    "constant.builtin",
    "constant.macro",
    "function.builtin",
    "function.call",
    "function.macro",
    "function.method",
    "function.method.call",
    "attribute.builtin",
    "module.builtin",
    "keyword.directive",
    "keyword.directive.define",
    "keyword.conditional",
    "keyword.conditional.ternary",
    "keyword.exception",
    "keyword.import",
    "keyword.operator",
    "keyword.coroutine",
    "keyword.function",
    "keyword.modifier",
    "keyword.repeat",
    "keyword.return",
    "keyword.debug",
    "keyword.type",
    "punctuation.delimiter",
    "punctuation.bracket",
    "punctuation.special",
    "comment",
    "string",
    "character",
    "number",
    "boolean",
    "keyword",
    "type",
    "variable",
    "constant",
    "function",
    "attribute",
    "module",
    "property",
    "operator",
    "punctuation",
    "markup",
    "diff",
    "label",
    "tag",
    "tag.builtin",
    "tag.attribute",
    "tag.delimiter",
];

const DOCUMENT_SYMBOL_QUERY: &str = include_str!("../queries/document_symbols.scm");
const DIAGNOSTICS_QUERY: &str = include_str!("../queries/diagnostics.scm");
const FOLDING_QUERY: &str = include_str!("../queries/folding_ranges.scm");
const GOTO_QUERY: &str = include_str!("../queries/goto_symbols.scm");
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
                            text_document_registration_options: {
                                TextDocumentRegistrationOptions {
                                    document_selector: Some(vec![DocumentFilter {
                                        language: Some("idl".to_string()),
                                        scheme: Some("file".to_string()),
                                        pattern: None,
                                    }]),
                                }
                            },
                            semantic_tokens_options: SemanticTokensOptions {
                                work_done_progress_options: WorkDoneProgressOptions::default(),
                                legend: SemanticTokensLegend {
                                    token_types: vec![
                                        SemanticTokenType::new("namespace"),
                                        SemanticTokenType::new("type"),
                                        SemanticTokenType::new("class"),
                                        SemanticTokenType::new("enum"),
                                        SemanticTokenType::new("interface"),
                                        SemanticTokenType::new("struct"),
                                        SemanticTokenType::new("typeParameter"),
                                        SemanticTokenType::new("parameter"),
                                        SemanticTokenType::new("variable"),
                                        SemanticTokenType::new("property"),
                                        SemanticTokenType::new("enumMember"),
                                        SemanticTokenType::new("event"),
                                        SemanticTokenType::new("function"),
                                        SemanticTokenType::new("method"),
                                        SemanticTokenType::new("macro"),
                                        SemanticTokenType::new("keyword"),
                                        SemanticTokenType::new("modifier"),
                                        SemanticTokenType::new("comment"),
                                        SemanticTokenType::new("string"),
                                        SemanticTokenType::new("number"),
                                        SemanticTokenType::new("regexp"),
                                        SemanticTokenType::new("operator"),
                                        SemanticTokenType::new("decorator"),
                                    ],
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
        self.on_change(TextDocumentChange {
            uri: params.text_document.uri,
            text: &params.text_document.text,
        })
        .await;
        debug!("file opened!");
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.on_change(TextDocumentChange {
            text: &params.content_changes[0].text,
            uri: params.text_document.uri,
        })
        .await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        if let Some((_, preview)) = self.preview_map.remove(&uri) {
            preview.stop();
        }
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let uri = params.text_document.uri.to_string();
        let semantic_tokens = self.build_semantic_tokens(&uri);
        if let Some(tokens) = semantic_tokens {
            return Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
                result_id: None,
                data: tokens,
            })));
        }
        Ok(None)
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        Ok(self.format_text(params))
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri.to_string();
        let rope = match self.document_map.get(&uri) {
            Some(rope) => rope,
            None => return Ok(None),
        };
        let text = rope.to_string();
        let symbols = build_document_symbols(&text, &rope);
        Ok(Some(DocumentSymbolResponse::Nested(symbols)))
    }

    async fn folding_range(&self, params: FoldingRangeParams) -> Result<Option<Vec<FoldingRange>>> {
        let uri = params.text_document.uri.to_string();
        let rope = match self.document_map.get(&uri) {
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
        let rope = match self.document_map.get(uri.as_str()) {
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
        let rope = match self.document_map.get(uri.as_str()) {
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
        let rope = match self.document_map.get(uri.as_str()) {
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
        let rope = match self.document_map.get(uri.as_str()) {
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
        let rope = match self.document_map.get(uri.as_str()) {
            Some(rope) => rope,
            None => return Ok(None),
        };
        let text = rope.to_string();
        let doc_hover = doc::build_hover(&text, &rope, &uri, position);
        let preview_hover = self.build_preview_hover(&text, &rope, &uri, position);
        Ok(merge_hover(doc_hover, preview_hover))
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = params.text_document.uri;
        let rope = match self.document_map.get(uri.as_str()) {
            Some(rope) => rope,
            None => return Ok(None),
        };
        let text = rope.to_string();
        let position = params.range.start;

        let relevant = http_client::document_is_http_relevant(&text);
        let on_interface = http_client::interface_at_position(&text, &rope, position);
        debug!(
            "code_action: uri={} relevant={} on_interface={}",
            uri,
            relevant,
            on_interface
        );
        if !relevant {
            return Ok(None);
        }
        if !on_interface {
            return Ok(None);
        }

        let is_running = self.preview_map.contains_key(uri.as_str());
        let (title, command) = if is_running {
            ("$(debug-stop) stop http client", http_client::CMD_STOP_HTTP_CLIENT)
        } else {
            ("$(play) start http client", http_client::CMD_START_HTTP_CLIENT)
        };

        let action = CodeAction {
            title: title.to_string(),
            kind: Some(CodeActionKind::QUICKFIX),
            command: Some(Command {
                title: title.to_string(),
                command: command.to_string(),
                arguments: Some(vec![serde_json::Value::String(uri.to_string())]),
            }),
            ..CodeAction::default()
        };

        Ok(Some(vec![CodeActionOrCommand::CodeAction(action)]))
    }

    async fn execute_command(
        &self,
        params: ExecuteCommandParams,
    ) -> Result<Option<serde_json::Value>> {
        let mut args = params.arguments;
        let uri_value = args.pop();
        let uri = match uri_value.and_then(|value| value.as_str().map(|s| s.to_string())) {
            Some(uri) => uri,
            None => {
                self.client
                    .show_message(MessageType::ERROR, "Missing document URI")
                    .await;
                return Ok(None);
            }
        };

        match params.command.as_str() {
            http_client::CMD_START_HTTP_CLIENT => {
                if self.preview_map.contains_key(&uri) {
                    return Ok(None);
                }
                let rope = match self.document_map.get(&uri) {
                    Some(rope) => rope,
                    None => {
                        self.client
                            .show_message(MessageType::ERROR, "Document not available")
                            .await;
                        return Ok(None);
                    }
                };
                let text = rope.to_string();
                match http_client::start_preview(&text).await {
                    Ok(preview) => {
                        let url = preview.scalar_url.clone();
                        self.preview_map.insert(uri, preview);
                        self.client
                            .show_message(MessageType::INFO, format!("HTTP client preview: {url}"))
                            .await;
                    }
                    Err(err) => {
                        self.client
                            .show_message(
                                MessageType::ERROR,
                                format!("Failed to start http client: {err}"),
                            )
                            .await;
                    }
                }
            }
            http_client::CMD_STOP_HTTP_CLIENT => {
                if let Some((_, preview)) = self.preview_map.remove(&uri) {
                    preview.stop();
                    self.client
                        .show_message(MessageType::INFO, "HTTP client preview stopped")
                        .await;
                }
            }
            _ => {}
        }

        Ok(None)
    }

    async fn code_lens(&self, params: CodeLensParams) -> Result<Option<Vec<CodeLens>>> {
        let uri = params.text_document.uri;
        let rope = match self.document_map.get(uri.as_str()) {
            Some(rope) => rope,
            None => return Ok(None),
        };
        let text = rope.to_string();
        if !http_client::document_is_http_relevant(&text) {
            return Ok(None);
        }

        let positions = http_client::interface_name_positions(&text, &rope);
        if positions.is_empty() {
            return Ok(None);
        }

        let is_running = self.preview_map.contains_key(uri.as_str());
        let mut lenses: Vec<CodeLens> = Vec::new();

        if is_running {
            if let Some(preview) = self.preview_map.get(uri.as_str()) {
                let open_title = "$(link-external) open scalar web";
                let stop_title = "$(debug-stop) stop http client";
                for position in positions {
                    lenses.push(CodeLens {
                        range: Range {
                            start: position,
                            end: position,
                        },
                        command: Some(Command {
                            title: open_title.to_string(),
                            command: "vscode.open".to_string(),
                            arguments: Some(vec![serde_json::Value::String(
                                preview.scalar_url.clone(),
                            )]),
                        }),
                        data: None,
                    });
                    lenses.push(CodeLens {
                        range: Range {
                            start: position,
                            end: position,
                        },
                        command: Some(Command {
                            title: stop_title.to_string(),
                            command: http_client::CMD_STOP_HTTP_CLIENT.to_string(),
                            arguments: Some(vec![serde_json::Value::String(uri.to_string())]),
                        }),
                        data: None,
                    });
                }
            }
        } else {
            let title = "$(play) start http client";
            let command = http_client::CMD_START_HTTP_CLIENT;
            for position in positions {
                lenses.push(CodeLens {
                    range: Range {
                        start: position,
                        end: position,
                    },
                    command: Some(Command {
                        title: title.to_string(),
                        command: command.to_string(),
                        arguments: Some(vec![serde_json::Value::String(uri.to_string())]),
                    }),
                    data: None,
                });
            }
        }

        Ok(Some(lenses))
    }

    async fn did_change_configuration(&self, _: DidChangeConfigurationParams) {
        debug!("configuration changed!");
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|_client| Backend {
        client: _client,
        document_map: DashMap::new(),
        semantic_tokens_map: DashMap::new(),
        preview_map: DashMap::new(),
    })
    .finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}

impl Backend {
    fn format_text(&self, params: DocumentFormattingParams) -> Option<Vec<TextEdit>> {
        let uri = params.text_document.uri.to_string();
        let rope = self.document_map.get(&uri)?;
        let formatted_text = match xidlc::fmt::format_idl_source(&rope.to_string()) {
            Ok(text) => text,
            Err(err) => {
                warn!("formatting failed for {}: {}", uri, err);
                return None;
            }
        };
        Some(vec![TextEdit {
            range: Range {
                start: Position::new(0, 0),
                end: Position::new(
                    rope.len_lines() as u32,
                    rope.line(rope.len_lines() - 1).len_chars() as u32,
                ),
            },
            new_text: formatted_text,
        }])
    }

    async fn on_change(&self, item: TextDocumentChange<'_>) {
        let uri = item.uri.to_string();
        let rope = Rope::from_str(item.text);
        self.document_map.insert(uri.clone(), rope);
        self.refresh_semantic_tokens(&uri, item.text);
        let diagnostics = build_diagnostics(item.text);
        self.client
            .publish_diagnostics(item.uri, diagnostics, None)
            .await;
        if let Some(preview) = self.preview_map.get(&uri) {
            preview.request_regen(item.text);
        }
    }

    fn build_preview_hover(
        &self,
        text: &str,
        rope: &Rope,
        uri: &Url,
        position: Position,
    ) -> Option<String> {
        let preview = self.preview_map.get(uri.as_str())?;
        if !http_client::interface_at_position(text, rope, position) {
            return None;
        }
        Some(format!("[Open Scalar UI]({})", preview.scalar_url))
    }

    fn build_semantic_tokens(&self, uri: &str) -> Option<Vec<SemanticToken>> {
        let tokens = self.semantic_tokens_map.get(uri)?;
        Some(tokens.clone())
    }

    fn refresh_semantic_tokens(&self, uri: &str, text: &str) {
        let rope = Rope::from_str(text);
        let tokens = build_highlight_tokens(text, &rope);
        self.semantic_tokens_map.insert(uri.to_string(), tokens);
    }
}

fn merge_hover(base: Option<Hover>, extra: Option<String>) -> Option<Hover> {
    match (base, extra) {
        (None, None) => None,
        (Some(hover), None) => Some(hover),
        (None, Some(extra)) => Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: extra,
            }),
            range: None,
        }),
        (Some(hover), Some(extra)) => {
            let mut value = hover_contents_to_markdown(&hover.contents);
            if !value.is_empty() {
                value.push_str("\n\n");
            }
            value.push_str(&extra);
            Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value,
                }),
                range: hover.range,
            })
        }
    }
}

fn hover_contents_to_markdown(contents: &HoverContents) -> String {
    match contents {
        HoverContents::Scalar(marked) => match marked {
            MarkedString::String(value) => value.clone(),
            MarkedString::LanguageString(value) => {
                format!("```{}\n{}\n```", value.language, value.value)
            }
        },
        HoverContents::Markup(content) => content.value.clone(),
        HoverContents::Array(values) => values
            .iter()
            .map(|value| match value {
                MarkedString::String(text) => text.clone(),
                MarkedString::LanguageString(value) => {
                    format!("```{}\n{}\n```", value.language, value.value)
                }
            })
            .collect::<Vec<_>>()
            .join("\n\n"),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
struct NodeKey {
    start: usize,
    end: usize,
}

#[derive(Clone, Debug)]
struct SymbolEntry {
    name: String,
    kind: SymbolKind,
    range: Range,
    selection_range: Range,
    start_byte: usize,
    interface_key: Option<NodeKey>,
    is_interface: bool,
    is_op: bool,
}

fn build_document_symbols(text: &str, rope: &Rope) -> Vec<DocumentSymbol> {
    let mut parser = Parser::new();
    if parser.set_language(&tree_sitter_idl::language()).is_err() {
        debug!("failed to set tree-sitter language for document symbols");
        return Vec::new();
    }
    let tree = match parser.parse(text, None) {
        Some(tree) => tree,
        None => {
            debug!("failed to parse document for document symbols");
            return Vec::new();
        }
    };

    let query = match Query::new(&tree_sitter_idl::language(), DOCUMENT_SYMBOL_QUERY) {
        Ok(query) => query,
        Err(err) => {
            debug!("failed to compile document symbol query: {err}");
            return Vec::new();
        }
    };

    let mut cursor = QueryCursor::new();
    let capture_names = query.capture_names();
    let mut entries: Vec<SymbolEntry> = Vec::new();

    let mut matches = cursor.matches(&query, tree.root_node(), text.as_bytes());
    while let Some(m) = matches.next() {
        let mut symbol_kind: Option<SymbolKind> = None;
        let mut symbol_node: Option<Node<'_>> = None;
        let mut name_node: Option<Node<'_>> = None;
        let mut is_interface = false;
        let mut is_op = false;

        for capture in m.captures {
            let capture_name = match capture_names.get(capture.index as usize) {
                Some(name) => *name,
                None => continue,
            };
            match capture_name {
                "struct" => {
                    symbol_kind = Some(SymbolKind::STRUCT);
                    symbol_node = Some(capture.node);
                }
                "struct.name" => name_node = Some(capture.node),
                "enum" => {
                    symbol_kind = Some(SymbolKind::ENUM);
                    symbol_node = Some(capture.node);
                }
                "enum.name" => name_node = Some(capture.node),
                "bitmask" => {
                    symbol_kind = Some(SymbolKind::ENUM);
                    symbol_node = Some(capture.node);
                }
                "bitmask.name" => name_node = Some(capture.node),
                "interface" => {
                    symbol_kind = Some(SymbolKind::INTERFACE);
                    symbol_node = Some(capture.node);
                    is_interface = true;
                }
                "interface.name" => name_node = Some(capture.node),
                "op" => {
                    symbol_kind = Some(SymbolKind::METHOD);
                    symbol_node = Some(capture.node);
                    is_op = true;
                }
                "op.name" => name_node = Some(capture.node),
                _ => {}
            }
        }

        let (symbol_kind, symbol_node, name_node) = match (symbol_kind, symbol_node, name_node) {
            (Some(kind), Some(symbol_node), Some(name_node)) => (kind, symbol_node, name_node),
            _ => continue,
        };
        let name = match name_node.utf8_text(text.as_bytes()) {
            Ok(name) => name.trim().to_string(),
            Err(_) => continue,
        };
        if name.is_empty() {
            continue;
        }

        let interface_key = if is_op {
            find_enclosing_interface_key(symbol_node)
        } else if is_interface {
            Some(node_key(symbol_node))
        } else {
            None
        };

        entries.push(SymbolEntry {
            name,
            kind: symbol_kind,
            range: node_range(symbol_node, rope),
            selection_range: node_range(name_node, rope),
            start_byte: symbol_node.start_byte(),
            interface_key,
            is_interface,
            is_op,
        });
    }

    build_document_symbol_tree(entries)
}

fn build_diagnostics(text: &str) -> Vec<Diagnostic> {
    let mut parser = Parser::new();
    if parser.set_language(&tree_sitter_idl::language()).is_err() {
        debug!("failed to set tree-sitter language for diagnostics");
        return Vec::new();
    }
    let tree = match parser.parse(text, None) {
        Some(tree) => tree,
        None => {
            debug!("failed to parse document for diagnostics");
            return Vec::new();
        }
    };

    let query = match Query::new(&tree_sitter_idl::language(), DIAGNOSTICS_QUERY) {
        Ok(query) => query,
        Err(err) => {
            debug!("failed to compile diagnostics query: {err}");
            return Vec::new();
        }
    };

    let mut cursor = QueryCursor::new();
    let capture_names = query.capture_names();
    let mut diagnostics = Vec::new();
    let mut captures = cursor.captures(&query, tree.root_node(), text.as_bytes());
    while let Some((m, idx)) = captures.next() {
        let capture = m.captures[*idx];
        let capture_name = match capture_names.get(capture.index as usize) {
            Some(name) => *name,
            None => continue,
        };
        if capture_name != "error" {
            continue;
        }
        let range = Range {
            start: Position::new(
                capture.node.start_position().row as u32,
                capture.node.start_position().column as u32,
            ),
            end: Position::new(
                capture.node.end_position().row as u32,
                capture.node.end_position().column as u32,
            ),
        };
        diagnostics.push(Diagnostic {
            range,
            severity: Some(DiagnosticSeverity::ERROR),
            code: None,
            code_description: None,
            source: Some("idl-language-server".to_string()),
            message: "Parse error".to_string(),
            related_information: None,
            tags: None,
            data: None,
        });
    }
    diagnostics
}

fn build_folding_ranges(text: &str, rope: &Rope) -> Vec<FoldingRange> {
    let mut parser = Parser::new();
    if parser.set_language(&tree_sitter_idl::language()).is_err() {
        debug!("failed to set tree-sitter language for folding ranges");
        return Vec::new();
    }
    let tree = match parser.parse(text, None) {
        Some(tree) => tree,
        None => {
            debug!("failed to parse document for folding ranges");
            return Vec::new();
        }
    };

    let query = match Query::new(&tree_sitter_idl::language(), FOLDING_QUERY) {
        Ok(query) => query,
        Err(err) => {
            debug!("failed to compile folding query: {err}");
            return Vec::new();
        }
    };

    let mut cursor = QueryCursor::new();
    let capture_names = query.capture_names();
    let mut ranges = Vec::new();
    let mut captures = cursor.captures(&query, tree.root_node(), text.as_bytes());
    while let Some((m, idx)) = captures.next() {
        let capture = m.captures[*idx];
        let capture_name = match capture_names.get(capture.index as usize) {
            Some(name) => *name,
            None => continue,
        };
        if capture_name != "fold" {
            continue;
        }
        let range = node_range(capture.node, rope);
        if range.start.line >= range.end.line {
            continue;
        }
        ranges.push(FoldingRange {
            start_line: range.start.line,
            start_character: Some(range.start.character),
            end_line: range.end.line,
            end_character: Some(range.end.character),
            kind: Some(FoldingRangeKind::Region),
            collapsed_text: None,
        });
    }
    ranges
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum GotoSymbolKind {
    Definition,
    Declaration,
}

#[derive(Clone, Debug)]
struct GotoSymbol {
    name: String,
    kind: GotoSymbolKind,
    range: Range,
    selection_range: Range,
}

fn build_goto_symbols(text: &str, rope: &Rope) -> Vec<GotoSymbol> {
    let mut parser = Parser::new();
    if parser.set_language(&tree_sitter_idl::language()).is_err() {
        debug!("failed to set tree-sitter language for goto symbols");
        return Vec::new();
    }
    let tree = match parser.parse(text, None) {
        Some(tree) => tree,
        None => {
            debug!("failed to parse document for goto symbols");
            return Vec::new();
        }
    };

    let query = match Query::new(&tree_sitter_idl::language(), GOTO_QUERY) {
        Ok(query) => query,
        Err(err) => {
            debug!("failed to compile goto query: {err}");
            return Vec::new();
        }
    };

    let mut cursor = QueryCursor::new();
    let capture_names = query.capture_names();
    let mut matches = cursor.matches(&query, tree.root_node(), text.as_bytes());
    let mut symbols = Vec::new();

    while let Some(m) = matches.next() {
        let mut symbol_kind: Option<GotoSymbolKind> = None;
        let mut symbol_node: Option<Node<'_>> = None;
        let mut name_node: Option<Node<'_>> = None;

        for capture in m.captures {
            let capture_name = match capture_names.get(capture.index as usize) {
                Some(name) => *name,
                None => continue,
            };
            match capture_name {
                "def" => {
                    symbol_kind = Some(GotoSymbolKind::Definition);
                    symbol_node = Some(capture.node);
                }
                "def.name" => name_node = Some(capture.node),
                "decl" => {
                    symbol_kind = Some(GotoSymbolKind::Declaration);
                    symbol_node = Some(capture.node);
                }
                "decl.name" => name_node = Some(capture.node),
                _ => {}
            }
        }

        let (symbol_kind, symbol_node, name_node) = match (symbol_kind, symbol_node, name_node) {
            (Some(kind), Some(symbol_node), Some(name_node)) => (kind, symbol_node, name_node),
            _ => continue,
        };
        let name = match name_node.utf8_text(text.as_bytes()) {
            Ok(name) => name.trim().to_string(),
            Err(_) => continue,
        };
        if name.is_empty() {
            continue;
        }

        symbols.push(GotoSymbol {
            name,
            kind: symbol_kind,
            range: node_range(symbol_node, rope),
            selection_range: node_range(name_node, rope),
        });
    }

    symbols
}

fn goto_definition_locations(
    symbols: &[GotoSymbol],
    uri: &Url,
    position: Position,
) -> Vec<Location> {
    let symbol = match symbol_at_position(symbols, position) {
        Some(symbol) => symbol,
        None => return Vec::new(),
    };
    if symbol.kind != GotoSymbolKind::Declaration {
        return Vec::new();
    }
    symbols
        .iter()
        .filter(|candidate| {
            candidate.kind == GotoSymbolKind::Definition && candidate.name == symbol.name
        })
        .map(|candidate| Location {
            uri: uri.clone(),
            range: candidate.selection_range,
        })
        .collect()
}

fn goto_declaration_locations(
    symbols: &[GotoSymbol],
    uri: &Url,
    position: Position,
) -> Vec<Location> {
    let symbol = match symbol_at_position(symbols, position) {
        Some(symbol) => symbol,
        None => return Vec::new(),
    };
    if symbol.kind != GotoSymbolKind::Declaration {
        return Vec::new();
    }
    vec![Location {
        uri: uri.clone(),
        range: symbol.selection_range,
    }]
}

fn reference_locations(symbols: &[GotoSymbol], uri: &Url, position: Position) -> Vec<Location> {
    let symbol = match symbol_at_position(symbols, position) {
        Some(symbol) => symbol,
        None => return Vec::new(),
    };
    if symbol.kind != GotoSymbolKind::Definition {
        return Vec::new();
    }
    symbols
        .iter()
        .filter(|candidate| {
            candidate.kind == GotoSymbolKind::Declaration && candidate.name == symbol.name
        })
        .map(|candidate| Location {
            uri: uri.clone(),
            range: candidate.selection_range,
        })
        .collect()
}

fn rename_workspace_edit(
    symbols: &[GotoSymbol],
    uri: &Url,
    position: Position,
    new_name: &str,
) -> Option<WorkspaceEdit> {
    let symbol = symbol_at_position(symbols, position)?;
    let mut ranges = Vec::new();

    match symbol.kind {
        GotoSymbolKind::Definition => {
            ranges.push(symbol.selection_range);
            for candidate in symbols.iter().filter(|candidate| {
                candidate.kind == GotoSymbolKind::Declaration && candidate.name == symbol.name
            }) {
                ranges.push(candidate.selection_range);
            }
        }
        GotoSymbolKind::Declaration => {
            for candidate in symbols.iter().filter(|candidate| {
                (candidate.kind == GotoSymbolKind::Definition
                    || candidate.kind == GotoSymbolKind::Declaration)
                    && candidate.name == symbol.name
            }) {
                ranges.push(candidate.selection_range);
            }
        }
    }

    if ranges.is_empty() {
        return None;
    }

    ranges.sort_by_key(|range| {
        (
            range.start.line,
            range.start.character,
            range.end.line,
            range.end.character,
        )
    });
    ranges.dedup_by(|a, b| a.start == b.start && a.end == b.end);

    let edits: Vec<TextEdit> = ranges
        .into_iter()
        .map(|range| TextEdit {
            range,
            new_text: new_name.to_string(),
        })
        .collect();

    if edits.is_empty() {
        return None;
    }

    let mut changes = std::collections::HashMap::new();
    changes.insert(uri.clone(), edits);
    Some(WorkspaceEdit {
        changes: Some(changes),
        document_changes: None,
        change_annotations: None,
    })
}

fn symbol_at_position<'a>(symbols: &'a [GotoSymbol], position: Position) -> Option<&'a GotoSymbol> {
    symbols
        .iter()
        .find(|symbol| position_in_range(position, symbol.selection_range))
}

fn position_in_range(position: Position, range: Range) -> bool {
    if position.line < range.start.line || position.line > range.end.line {
        return false;
    }
    if position.line == range.start.line && position.character < range.start.character {
        return false;
    }
    if position.line == range.end.line && position.character > range.end.character {
        return false;
    }
    true
}

fn build_document_symbol_tree(entries: Vec<SymbolEntry>) -> Vec<DocumentSymbol> {
    let mut interfaces: Vec<(NodeKey, usize, DocumentSymbol)> = Vec::new();
    let mut interface_index: std::collections::HashMap<NodeKey, usize> =
        std::collections::HashMap::new();
    let mut top_level: Vec<(usize, DocumentSymbol)> = Vec::new();

    for entry in entries.iter().filter(|entry| entry.is_interface) {
        let symbol = DocumentSymbol {
            name: entry.name.clone(),
            detail: None,
            kind: entry.kind,
            tags: None,
            deprecated: None,
            range: entry.range,
            selection_range: entry.selection_range,
            children: Some(Vec::new()),
        };
        let key = entry.interface_key.unwrap_or(NodeKey {
            start: entry.start_byte,
            end: entry.start_byte,
        });
        let idx = interfaces.len();
        interfaces.push((key, entry.start_byte, symbol));
        interface_index.insert(key, idx);
    }

    for entry in entries.into_iter().filter(|entry| !entry.is_interface) {
        let symbol = DocumentSymbol {
            name: entry.name,
            detail: None,
            kind: entry.kind,
            tags: None,
            deprecated: None,
            range: entry.range,
            selection_range: entry.selection_range,
            children: None,
        };

        if entry.is_op {
            if let Some(key) = entry.interface_key {
                if let Some(&idx) = interface_index.get(&key) {
                    if let Some(children) = interfaces[idx].2.children.as_mut() {
                        children.push(symbol);
                        continue;
                    }
                }
            }
        }

        top_level.push((entry.start_byte, symbol));
    }

    let mut merged: Vec<(usize, DocumentSymbol)> = interfaces
        .into_iter()
        .map(|(_, start, symbol)| (start, symbol))
        .collect();
    merged.extend(top_level);
    merged.sort_by_key(|(start, _)| *start);

    merged.into_iter().map(|(_, symbol)| symbol).collect()
}

fn find_enclosing_interface_key(node: Node<'_>) -> Option<NodeKey> {
    let mut current = node.parent();
    while let Some(parent) = current {
        if parent.kind() == "interface_def" || parent.kind() == "interface_forward_dcl" {
            return Some(node_key(parent));
        }
        current = parent.parent();
    }
    None
}

fn node_key(node: Node<'_>) -> NodeKey {
    NodeKey {
        start: node.start_byte(),
        end: node.end_byte(),
    }
}

fn node_range(node: Node<'_>, rope: &Rope) -> Range {
    Range {
        start: byte_to_position(rope, node.start_byte()),
        end: byte_to_position(rope, node.end_byte()),
    }
}

fn byte_to_position(rope: &Rope, byte: usize) -> Position {
    let line = rope.byte_to_line(byte);
    let column = rope
        .byte_to_char(byte)
        .saturating_sub(rope.line_to_char(line));
    Position::new(line as u32, column as u32)
}

fn build_highlight_tokens(text: &str, rope: &Rope) -> Vec<SemanticToken> {
    let mut config = match HighlightConfiguration::new(
        tree_sitter_idl::language(),
        "idl",
        tree_sitter_idl::HIGHLIGHTS_QUERY,
        "",
        "",
    ) {
        Ok(config) => config,
        Err(err) => {
            debug!("failed to create highlight config: {err}");
            return Vec::new();
        }
    };
    config.configure(HIGHLIGHT_NAMES);

    let mut highlighter = Highlighter::new();
    let events = match highlighter.highlight(&config, text.as_bytes(), None, |_| None) {
        Ok(events) => events,
        Err(err) => {
            debug!("failed to highlight source: {err}");
            return Vec::new();
        }
    };

    let mut highlight_stack: Vec<usize> = Vec::new();
    let mut raw_spans: Vec<(usize, usize, usize)> = Vec::new();
    let mut highlight_events = 0usize;

    for event in events {
        match event {
            Ok(HighlightEvent::HighlightStart(highlight)) => {
                highlight_events += 1;
                highlight_stack.push(highlight.0);
            }
            Ok(HighlightEvent::HighlightEnd) => {
                let _ = highlight_stack.pop();
            }
            Ok(HighlightEvent::Source { start, end }) => {
                if let Some(&highlight_index) = highlight_stack.last() {
                    if end > start {
                        raw_spans.push((start, end, highlight_index));
                    }
                }
            }
            Err(err) => {
                debug!("highlight event error: {err}");
                return Vec::new();
            }
        }
    }
    if highlight_events == 0 {
        debug!("no highlight events produced for source");
    }

    let mut incomplete_tokens: Vec<(u32, u32, u32, u32)> = Vec::new();
    for (start, end, highlight_index) in raw_spans {
        let capture_name = match HIGHLIGHT_NAMES.get(highlight_index) {
            Some(name) => *name,
            None => continue,
        };
        let token_type = match capture_to_token_type(capture_name) {
            Some(token_type) => token_type,
            None => continue,
        };

        let mut cur = start;
        while cur < end {
            let line = match rope.try_byte_to_line(cur) {
                Ok(line) => line,
                Err(_) => break,
            };
            let line_start = match rope.try_line_to_byte(line) {
                Ok(byte) => byte,
                Err(_) => break,
            };
            let line_len = rope.line(line).len_bytes();
            let line_end = line_start + line_len;
            let seg_end = end.min(line_end);
            let char_offset = cur.saturating_sub(line_start);
            let length = seg_end.saturating_sub(cur);
            if length > 0 {
                incomplete_tokens.push((
                    line as u32,
                    char_offset as u32,
                    length as u32,
                    token_type,
                ));
            }
            cur = seg_end;
        }
    }

    incomplete_tokens.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

    let mut tokens = Vec::with_capacity(incomplete_tokens.len());
    let mut pre_line: u32 = 0;
    let mut pre_start: u32 = 0;

    for (line, start, length, token_type) in incomplete_tokens {
        let delta_line = line - pre_line;
        let delta_start = if delta_line == 0 {
            start - pre_start
        } else {
            start
        };
        tokens.push(SemanticToken {
            delta_line,
            delta_start,
            length,
            token_type,
            token_modifiers_bitset: 0,
        });
        pre_line = line;
        pre_start = start;
    }

    tokens
}

fn capture_to_token_type(capture: &str) -> Option<u32> {
    match capture {
        "comment.documentation"
        | "comment.error"
        | "comment.warning"
        | "comment.todo"
        | "comment.note"
        | "comment" => Some(TOKEN_TYPE_COMMENT),
        "string.documentation"
        | "string.escape"
        | "string.special"
        | "string.special.symbol"
        | "string.special.path"
        | "string.special.url"
        | "string" => Some(TOKEN_TYPE_STRING),
        "string.regexp" => Some(TOKEN_TYPE_REGEXP),
        "character.special" | "character" => Some(TOKEN_TYPE_STRING),
        "number.float" | "number" | "boolean" => Some(TOKEN_TYPE_NUMBER),
        "keyword.modifier" => Some(TOKEN_TYPE_MODIFIER),
        "keyword.directive" => Some(TOKEN_TYPE_MACRO),
        "keyword.directive.define"
        | "keyword.conditional"
        | "keyword.conditional.ternary"
        | "keyword.exception"
        | "keyword.import"
        | "keyword.operator"
        | "keyword.coroutine"
        | "keyword.function"
        | "keyword.repeat"
        | "keyword.return"
        | "keyword.debug"
        | "keyword.type"
        | "keyword" => Some(TOKEN_TYPE_KEYWORD),
        "type.builtin" | "type.definition" | "type" | "tag" | "tag.builtin" | "tag.attribute"
        | "tag.delimiter" => Some(TOKEN_TYPE_TYPE),
        "function.macro" => Some(TOKEN_TYPE_MACRO),
        "function.method" | "function.method.call" => Some(TOKEN_TYPE_METHOD),
        "function.builtin" | "function.call" | "function" => Some(TOKEN_TYPE_FUNCTION),
        "variable.parameter" | "variable.parameter.builtin" => Some(TOKEN_TYPE_PARAMETER),
        "variable.member" | "property" => Some(TOKEN_TYPE_PROPERTY),
        "variable.builtin" | "variable" | "label" => Some(TOKEN_TYPE_VARIABLE),
        "constant.macro" => Some(TOKEN_TYPE_MACRO),
        "constant.builtin" | "constant" => Some(TOKEN_TYPE_ENUM_MEMBER),
        "operator"
        | "punctuation.delimiter"
        | "punctuation.bracket"
        | "punctuation.special"
        | "punctuation" => Some(TOKEN_TYPE_OPERATOR),
        "module.builtin" | "module" => Some(TOKEN_TYPE_NAMESPACE),
        "attribute.builtin" | "attribute" => Some(TOKEN_TYPE_DECORATOR),
        "markup" | "diff" => None,
        _ => None,
    }
}

struct TextDocumentChange<'a> {
    uri: Url,
    text: &'a str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_symbols_builds_hierarchy() {
        let source = r#"
interface RobotControl {
    void command(Command com);
};

struct Hello {
    long value;
};

enum Color {
    RED,
    GREEN
};

bitmask Flags {
    @position(0) a,
};
"#;
        let rope = Rope::from_str(source);
        let symbols = build_document_symbols(source, &rope);

        let interface = symbols.iter().find(|symbol| symbol.name == "RobotControl");
        assert!(interface.is_some());

        let interface = interface.unwrap();
        let children = interface.children.as_ref().unwrap();
        assert!(children.iter().any(|child| child.name == "command"));

        assert!(symbols.iter().any(|symbol| symbol.name == "Hello"));
        assert!(symbols.iter().any(|symbol| symbol.name == "Color"));
        assert!(symbols.iter().any(|symbol| symbol.name == "Flags"));
    }

    #[test]
    fn diagnostics_collects_error_nodes() {
        let source = "interface A { void m( }";
        let diagnostics = build_diagnostics(source);
        assert!(!diagnostics.is_empty());
    }

    #[test]
    fn folding_ranges_include_interface_and_module() {
        let source = r#"
module M {
    interface A {
        void m();
    };
};
"#;
        let rope = Rope::from_str(source);
        let ranges = build_folding_ranges(source, &rope);
        assert!(!ranges.is_empty());
    }

    #[test]
    fn goto_definition_resolves_from_declaration() {
        let source = r#"
struct Foo {
    long value;
};

interface Bar {
    void takeFoo(Foo f);
};
"#;
        let rope = Rope::from_str(source);
        let symbols = build_goto_symbols(source, &rope);
        let decl = symbols
            .iter()
            .find(|symbol| symbol.kind == GotoSymbolKind::Declaration && symbol.name == "Foo")
            .expect("declaration not found");
        let locations = goto_definition_locations(
            &symbols,
            &Url::parse("file:///test.idl").unwrap(),
            decl.selection_range.start,
        );
        assert!(!locations.is_empty());
    }

    #[test]
    fn references_resolve_from_definition_only() {
        let source = r#"
struct Foo {
    long value;
};

interface Bar {
    void takeFoo(Foo f);
};
"#;
        let rope = Rope::from_str(source);
        let symbols = build_goto_symbols(source, &rope);
        let def = symbols
            .iter()
            .find(|symbol| symbol.kind == GotoSymbolKind::Definition && symbol.name == "Foo")
            .expect("definition not found");
        let locations = reference_locations(
            &symbols,
            &Url::parse("file:///test.idl").unwrap(),
            def.selection_range.start,
        );
        assert!(!locations.is_empty());
    }

    #[test]
    fn rename_produces_edits_for_definition_and_references() {
        let source = r#"
struct Foo {
    long value;
};

interface Bar {
    void takeFoo(Foo f);
};
"#;
        let rope = Rope::from_str(source);
        let symbols = build_goto_symbols(source, &rope);
        let def = symbols
            .iter()
            .find(|symbol| symbol.kind == GotoSymbolKind::Definition && symbol.name == "Foo")
            .expect("definition not found");
        let edit = rename_workspace_edit(
            &symbols,
            &Url::parse("file:///test.idl").unwrap(),
            def.selection_range.start,
            "FooRenamed",
        )
        .expect("rename edit missing");
        let changes = edit.changes.expect("missing changes");
        let edits = changes
            .get(&Url::parse("file:///test.idl").unwrap())
            .expect("missing edits");
        assert!(!edits.is_empty());
    }

    #[test]
    fn hover_template_renders_http() {
        let template = doc::load_hover_template("http.md").expect("missing hover template");
        let mut env = minijinja::Environment::new();
        env.add_template("hover", &template)
            .expect("failed to add hover template");
        let ctx = json!({
            "doc": {
                "name": "http",
                "path": "http.md",
            },
            "symbol_name": "http",
            "hir": json!(null),
        });
        let rendered = env
            .get_template("hover")
            .expect("template missing")
            .render(ctx)
            .expect("render failed");
        assert!(rendered.contains("HTTP Mapping"));
    }
}
