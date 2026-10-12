use log::{debug, warn};
use ropey::Rope;
use tower_lsp::lsp_types::*;

use crate::analysis::{build_diagnostics, build_highlight_tokens, byte_to_position};
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
    debug!("formatting document: {}", uri);
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
            end: byte_to_position(&rope, rope.len_bytes()),
        },
        new_text: formatted_text,
    }])
}

pub(crate) async fn on_change(ctx: &AppContext, item: TextDocumentChange<'_>) {
    let uri = item.uri.to_string();
    debug!("updating document content for: {}", uri);
    let rope = Rope::from_str(item.text);
    ctx.document_map.insert(uri.clone(), rope);
    refresh_semantic_tokens(ctx, &uri, item.text);
    let diagnostics = build_diagnostics(item.text);
    ctx.client
        .publish_diagnostics(item.uri, diagnostics, None)
        .await;
    if let Some(preview) = ctx.preview_map.get(&uri) {
        debug!("requesting preview regeneration for: {}", uri);
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

#[cfg(test)]
mod tests {
    #[test]
    fn formatter_accepts_new_media_type_and_wire_naming_annotations() {
        let source = r#"@request(format = "json")
@response(format = "msgpack")
interface Store {
    @get(path = "/items/{id}")
    @request(format = "urlencoded")
    string fetch(@path @rename("id") in string itemId);
};

@http(304) exception Cached { @body string msg; };

@rename_all(rule = "snake_case")
struct Record {
    @rename("item_id") @skip
    long itemId;
};
"#;
        let formatted = xidlc::fmt::format_idl_source(source)
            .expect("formatter should accept the new annotations");
        assert!(formatted.contains("@request(format = \"json\")"));
        assert!(formatted.contains("@http(304)"));
        assert!(formatted.contains("@rename(\"item_id\")"));
    }
}
