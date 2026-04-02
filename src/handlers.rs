use log::{debug, warn};
use ropey::Rope;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use crate::constants::{
    COMMAND_INSPECT_HIR, COMMAND_INSPECT_TYPEDAST, COMMAND_VSCODE_OPEN, MSG_DOCUMENT_NOT_AVAILABLE,
    MSG_MISSING_DOCUMENT_URI, TITLE_OPEN_HTTP_PREVIEW, TITLE_START_HTTP_PREVIEW,
    TITLE_STOP_HTTP_CLIENT, TITLE_STOP_HTTP_PREVIEW,
};
use crate::context::AppContext;
use crate::doc::{self, InspectTarget};
use crate::http_client;

pub(crate) async fn refresh_code_lens(ctx: &AppContext) {
    if let Err(err) = ctx.client.code_lens_refresh().await {
        warn!("failed to refresh code lens: {}", err);
    }
}

pub(crate) fn merge_hover(base: Option<Hover>, extra: Option<String>) -> Option<Hover> {
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

pub(crate) fn build_preview_hover(
    ctx: &AppContext,
    text: &str,
    rope: &Rope,
    uri: &Url,
    position: Position,
) -> Option<String> {
    let preview = ctx.preview_map.get(uri.as_str())?;
    if !http_client::interface_name_at_position(text, rope, position) {
        return None;
    }
    Some(format!("[Open Scalar UI]({})", preview.scalar_url))
}

pub(crate) fn hover(ctx: &AppContext, uri: &Url, position: Position) -> Option<Hover> {
    let rope = ctx.document_map.get(uri.as_str())?;
    let text = rope.to_string();
    let doc_hover = doc::build_hover(&text, &rope, uri, position);
    let preview_hover = build_preview_hover(ctx, &text, &rope, uri, position);
    merge_hover(doc_hover, preview_hover)
}

pub(crate) fn code_action(
    ctx: &AppContext,
    uri: &Url,
    params: &CodeActionParams,
) -> Option<CodeActionResponse> {
    let rope = ctx.document_map.get(uri.as_str())?;
    let text = rope.to_string();
    let position = params.range.start;
    let mut actions = suggestion_code_actions(uri, &params.context.diagnostics);

    let relevant = http_client::document_is_http_relevant(&text);
    let on_interface = http_client::interface_name_at_position(&text, &rope, position);
    debug!(
        "code_action: uri={} relevant={} on_interface={}",
        uri, relevant, on_interface
    );
    if relevant && on_interface {
        let is_running = ctx.preview_map.contains_key(uri.as_str());
        let (title, command) = if is_running {
            (TITLE_STOP_HTTP_PREVIEW, http_client::CMD_STOP_HTTP_CLIENT)
        } else {
            (TITLE_START_HTTP_PREVIEW, http_client::CMD_START_HTTP_CLIENT)
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

        actions.push(CodeActionOrCommand::CodeAction(action));
    }

    (!actions.is_empty()).then_some(actions)
}

fn suggestion_code_actions(uri: &Url, diagnostics: &[Diagnostic]) -> CodeActionResponse {
    diagnostics
        .iter()
        .filter_map(|diagnostic| {
            suggestion_from_message(&diagnostic.message).map(|suggestion| (diagnostic, suggestion))
        })
        .map(|(diagnostic, suggestion)| {
            CodeActionOrCommand::CodeAction(CodeAction {
                title: format!("Change to `{suggestion}`"),
                kind: Some(CodeActionKind::QUICKFIX),
                diagnostics: Some(vec![diagnostic.clone()]),
                edit: Some(WorkspaceEdit {
                    changes: Some(std::collections::HashMap::from([(
                        uri.clone(),
                        vec![TextEdit {
                            range: diagnostic.range,
                            new_text: suggestion,
                        }],
                    )])),
                    ..WorkspaceEdit::default()
                }),
                is_preferred: Some(true),
                ..CodeAction::default()
            })
        })
        .collect()
}

fn suggestion_from_message(message: &str) -> Option<String> {
    let prefix = "Unknown type in current file: ";
    let marker = ". Did you mean ";
    let rest = message.strip_prefix(prefix)?;
    let (_, suggestion_with_q) = rest.split_once(marker)?;
    suggestion_with_q.strip_suffix('?').map(str::to_string)
}

pub(crate) async fn execute_command(
    ctx: &AppContext,
    params: ExecuteCommandParams,
) -> Result<Option<serde_json::Value>> {
    let mut args = params.arguments;
    let uri_value = args.pop();
    let uri = match uri_value.and_then(|value| value.as_str().map(|s| s.to_string())) {
        Some(uri) => uri,
        None => {
            ctx.client
                .show_message(MessageType::ERROR, MSG_MISSING_DOCUMENT_URI)
                .await;
            return Ok(None);
        }
    };

    match params.command.as_str() {
        COMMAND_INSPECT_HIR | COMMAND_INSPECT_TYPEDAST => {
            let rope = match ctx.document_map.get(&uri) {
                Some(rope) => rope,
                None => {
                    ctx.client
                        .show_message(MessageType::ERROR, MSG_DOCUMENT_NOT_AVAILABLE)
                        .await;
                    return Ok(None);
                }
            };
            let text = rope.to_string();
            let target = if params.command == COMMAND_INSPECT_HIR {
                InspectTarget::Hir
            } else {
                InspectTarget::TypedAst
            };
            return Ok(Some(doc::build_inspect_value(&text, target)));
        }
        http_client::CMD_START_HTTP_CLIENT => {
            if ctx.preview_map.contains_key(&uri) {
                return Ok(None);
            }
            let rope = match ctx.document_map.get(&uri) {
                Some(rope) => rope,
                None => {
                    ctx.client
                        .show_message(MessageType::ERROR, MSG_DOCUMENT_NOT_AVAILABLE)
                        .await;
                    return Ok(None);
                }
            };
            let text = rope.to_string();
            match http_client::start_preview(&text).await {
                Ok(preview) => {
                    ctx.preview_map.insert(uri, preview);
                    refresh_code_lens(ctx).await;
                }
                Err(err) => {
                    ctx.client
                        .show_message(
                            MessageType::ERROR,
                            format!("Failed to start http client: {err}"),
                        )
                        .await;
                }
            }
        }
        http_client::CMD_STOP_HTTP_CLIENT => {
            if let Some((_, preview)) = ctx.preview_map.remove(&uri) {
                preview.stop();
                refresh_code_lens(ctx).await;
            }
        }
        _ => {}
    }

    Ok(None)
}

pub(crate) fn code_lens(ctx: &AppContext, uri: &Url) -> Option<Vec<CodeLens>> {
    let rope = ctx.document_map.get(uri.as_str())?;
    let text = rope.to_string();
    if !http_client::document_is_http_relevant(&text) {
        return None;
    }

    let positions = http_client::interface_name_positions(&text, &rope);
    if positions.is_empty() {
        return None;
    }

    let is_running = ctx.preview_map.contains_key(uri.as_str());
    let mut lenses: Vec<CodeLens> = Vec::new();

    if is_running {
        if let Some(preview) = ctx.preview_map.get(uri.as_str()) {
            for position in positions {
                lenses.push(CodeLens {
                    range: Range {
                        start: position,
                        end: position,
                    },
                    command: Some(Command {
                        title: TITLE_OPEN_HTTP_PREVIEW.to_string(),
                        command: COMMAND_VSCODE_OPEN.to_string(),
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
                        title: TITLE_STOP_HTTP_CLIENT.to_string(),
                        command: http_client::CMD_STOP_HTTP_CLIENT.to_string(),
                        arguments: Some(vec![serde_json::Value::String(uri.to_string())]),
                    }),
                    data: None,
                });
            }
        }
    } else {
        for position in positions {
            lenses.push(CodeLens {
                range: Range {
                    start: position,
                    end: position,
                },
                command: Some(Command {
                    title: TITLE_START_HTTP_PREVIEW.to_string(),
                    command: http_client::CMD_START_HTTP_CLIENT.to_string(),
                    arguments: Some(vec![serde_json::Value::String(uri.to_string())]),
                }),
                data: None,
            });
        }
    }

    Some(lenses)
}

#[cfg(test)]
mod tests {
    use super::{suggestion_code_actions, suggestion_from_message};
    use tower_lsp::lsp_types::{
        CodeActionOrCommand, Diagnostic, DiagnosticSeverity, Position, Range, Url,
    };

    #[test]
    fn extracts_suggestion_from_unknown_type_message() {
        let message = "Unknown type in current file: uant8. Did you mean uint8?";
        assert_eq!(suggestion_from_message(message).as_deref(), Some("uint8"));
    }

    #[test]
    fn ignores_unknown_type_message_without_suggestion() {
        let message = "Unknown type in current file: Foo";
        assert_eq!(suggestion_from_message(message), None);
    }

    #[test]
    fn builds_quickfix_for_suggested_type_diagnostic() {
        let uri = Url::parse("file:///test.idl").unwrap();
        let diagnostic = Diagnostic {
            range: Range {
                start: Position::new(3, 17),
                end: Position::new(3, 22),
            },
            severity: Some(DiagnosticSeverity::WARNING),
            code: None,
            code_description: None,
            source: Some("idl".to_string()),
            message: "Unknown type in current file: uant8. Did you mean uint8?".to_string(),
            related_information: None,
            tags: None,
            data: None,
        };

        let actions = suggestion_code_actions(&uri, &[diagnostic.clone()]);
        assert_eq!(actions.len(), 1);

        let CodeActionOrCommand::CodeAction(action) = &actions[0] else {
            panic!("expected code action");
        };

        assert_eq!(action.title, "Change to `uint8`");
        assert_eq!(action.diagnostics.as_ref(), Some(&vec![diagnostic.clone()]));
        let edit = action.edit.as_ref().expect("workspace edit");
        let changes = edit.changes.as_ref().expect("changes");
        let text_edits = changes.get(&uri).expect("uri edits");
        assert_eq!(text_edits.len(), 1);
        assert_eq!(text_edits[0].range, diagnostic.range);
        assert_eq!(text_edits[0].new_text, "uint8");
    }
}
