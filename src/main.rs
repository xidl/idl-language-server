use std::env;

use dashmap::DashMap;
use log::{debug, warn};
use ropey::Rope;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{LanguageServer, LspService, Server};
use tree_sitter_highlight::{HighlightConfiguration, HighlightEvent, Highlighter};

#[derive(Debug)]
struct Backend {
    document_map: DashMap<String, Rope>,
    semantic_tokens_map: DashMap<String, Vec<SemanticToken>>,
}

const TOKEN_TYPE_COMMENT: u32 = 0;
const TOKEN_TYPE_STRING: u32 = 1;
const TOKEN_TYPE_NUMBER: u32 = 2;
const TOKEN_TYPE_KEYWORD: u32 = 3;
const TOKEN_TYPE_TYPE: u32 = 4;
const TOKEN_TYPE_FUNCTION: u32 = 5;
const TOKEN_TYPE_VARIABLE: u32 = 6;
const TOKEN_TYPE_PARAMETER: u32 = 7;
const TOKEN_TYPE_PROPERTY: u32 = 8;
const TOKEN_TYPE_ENUM_MEMBER: u32 = 9;
const TOKEN_TYPE_OPERATOR: u32 = 10;
const TOKEN_TYPE_NAMESPACE: u32 = 11;
const TOKEN_TYPE_DECORATOR: u32 = 12;

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
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensRegistrationOptions(
                        SemanticTokensRegistrationOptions {
                            text_document_registration_options: {
                                TextDocumentRegistrationOptions {
                                    document_selector: Some(vec![DocumentFilter {
                                        language: Some("l".to_string()),
                                        scheme: Some("file".to_string()),
                                        pattern: None,
                                    }]),
                                }
                            },
                            semantic_tokens_options: SemanticTokensOptions {
                                work_done_progress_options: WorkDoneProgressOptions::default(),
                                legend: SemanticTokensLegend {
                                    token_types: vec![
                                        SemanticTokenType::new("comment"),
                                        SemanticTokenType::new("string"),
                                        SemanticTokenType::new("number"),
                                        SemanticTokenType::new("keyword"),
                                        SemanticTokenType::new("type"),
                                        SemanticTokenType::new("function"),
                                        SemanticTokenType::new("variable"),
                                        SemanticTokenType::new("parameter"),
                                        SemanticTokenType::new("property"),
                                        SemanticTokenType::new("enumMember"),
                                        SemanticTokenType::new("operator"),
                                        SemanticTokenType::new("namespace"),
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
            uri: params.text_document.uri.to_string(),
            text: &params.text_document.text,
        })
        .await;
        debug!("file opened!");
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.on_change(TextDocumentChange {
            text: &params.content_changes[0].text,
            uri: params.text_document.uri.to_string(),
        })
        .await;
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
        document_map: DashMap::new(),
        semantic_tokens_map: DashMap::new(),
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
        let rope = Rope::from_str(item.text);
        self.document_map.insert(item.uri.clone(), rope);
        self.refresh_semantic_tokens(&item.uri, item.text);
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
    if capture.starts_with("comment") {
        return Some(TOKEN_TYPE_COMMENT);
    }
    if capture.starts_with("string") || capture.starts_with("character") {
        return Some(TOKEN_TYPE_STRING);
    }
    if capture.starts_with("number") || capture == "boolean" {
        return Some(TOKEN_TYPE_NUMBER);
    }
    if capture.starts_with("keyword") {
        return Some(TOKEN_TYPE_KEYWORD);
    }
    if capture.starts_with("type") || capture.starts_with("tag") {
        return Some(TOKEN_TYPE_TYPE);
    }
    if capture.starts_with("function") {
        return Some(TOKEN_TYPE_FUNCTION);
    }
    if capture.starts_with("variable.parameter") {
        return Some(TOKEN_TYPE_PARAMETER);
    }
    if capture.starts_with("variable.member") || capture.starts_with("property") {
        return Some(TOKEN_TYPE_PROPERTY);
    }
    if capture.starts_with("variable") || capture == "label" {
        return Some(TOKEN_TYPE_VARIABLE);
    }
    if capture.starts_with("constant") {
        return Some(TOKEN_TYPE_ENUM_MEMBER);
    }
    if capture.starts_with("operator") || capture.starts_with("punctuation") {
        return Some(TOKEN_TYPE_OPERATOR);
    }
    if capture.starts_with("module") {
        return Some(TOKEN_TYPE_NAMESPACE);
    }
    if capture.starts_with("attribute") {
        return Some(TOKEN_TYPE_DECORATOR);
    }
    None
}

struct TextDocumentChange<'a> {
    uri: String,
    text: &'a str,
}
