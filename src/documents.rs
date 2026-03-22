use log::warn;
use ropey::Rope;
use tower_lsp::lsp_types::*;

use crate::analysis::{build_diagnostics, build_highlight_tokens};
use crate::context::AppContext;

pub(crate) struct TextDocumentChange<'a> {
    pub(crate) uri: Url,
    pub(crate) text: &'a str,
}

pub(crate) fn format_text(
    ctx: &AppContext,
    params: DocumentFormattingParams,
) -> Option<Vec<TextEdit>> {
    let uri = params.text_document.uri.to_string();
    let rope = ctx.document_map.get(&uri)?;
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

pub(crate) async fn on_change(ctx: &AppContext, item: TextDocumentChange<'_>) {
    let uri = item.uri.to_string();
    let rope = Rope::from_str(item.text);
    ctx.document_map.insert(uri.clone(), rope);
    refresh_semantic_tokens(ctx, &uri, item.text);
    let diagnostics = build_diagnostics(item.text);
    ctx.client
        .publish_diagnostics(item.uri, diagnostics, None)
        .await;
    if let Some(preview) = ctx.preview_map.get(&uri) {
        preview.request_regen(item.text);
    }
}

pub(crate) fn semantic_tokens(ctx: &AppContext, uri: &str) -> Option<Vec<SemanticToken>> {
    let tokens = ctx.semantic_tokens_map.get(uri)?;
    Some(tokens.clone())
}

pub(crate) fn refresh_semantic_tokens(ctx: &AppContext, uri: &str, text: &str) {
    let rope = Rope::from_str(text);
    let tokens = build_highlight_tokens(text, &rope);
    ctx.semantic_tokens_map.insert(uri.to_string(), tokens);
}
