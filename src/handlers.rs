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
    position: Position,
) -> Option<CodeActionResponse> {
    let rope = ctx.document_map.get(uri.as_str())?;
    let text = rope.to_string();

    let relevant = http_client::document_is_http_relevant(&text);
    let on_interface = http_client::interface_name_at_position(&text, &rope, position);
    debug!(
        "code_action: uri={} relevant={} on_interface={}",
        uri, relevant, on_interface
    );
    if !relevant || !on_interface {
        return None;
    }

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

    Some(vec![CodeActionOrCommand::CodeAction(action)])
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
