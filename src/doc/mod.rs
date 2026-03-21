use std::sync::Arc;

use minijinja::Environment as JinjaEnvironment;
use minijinja::value::Value as JinjaValue;
use ropey::Rope;
use rust_embed::RustEmbed;
use serde_json::json;
use tower_lsp::lsp_types::{
    Hover, HoverContents, MarkedString, MarkupContent, MarkupKind, Position, Url,
};
use tree_sitter::{Parser, Query, QueryCursor, StreamingIterator};
use xidl_parser::parser::parser_text;

use super::{GotoSymbol, GotoSymbolKind, build_goto_symbols, node_range, position_in_range};

pub(crate) mod http;

const HOVER_QUERY: &str = include_str!("../../queries/hover_docs.scm");

#[derive(RustEmbed)]
#[folder = "docs/hover"]
struct HoverDocs;

pub(super) fn build_hover(text: &str, rope: &Rope, uri: &Url, position: Position) -> Option<Hover> {
    let (doc_name, template_path) = match hover_template_at_position(text, rope, position) {
        Some(hit) => hit,
        None => return None,
    };

    let template = match load_hover_template(&template_path) {
        Some(template) => template,
        None => {
            return Some(Hover {
                contents: HoverContents::Scalar(MarkedString::String(format!(
                    "No documentation template found for `{doc_name}`"
                ))),
                range: None,
            });
        }
    };

    let symbols = Arc::new(build_goto_symbols(text, rope));
    let hir_value = match parser_text(text) {
        Ok(spec) => {
            let hir = xidl_parser::hir::Specification::from(spec);
            serde_json::to_value(hir).unwrap_or_else(|_| json!(null))
        }
        Err(_) => json!(null),
    };

    let uri_string = uri.to_string();
    let symbol_uri = uri_string.clone();
    let reference_uri = uri_string.clone();

    let mut env = JinjaEnvironment::new();
    let symbols_for_symbol = Arc::clone(&symbols);
    env.add_function("find_symbol", move |name: String| {
        JinjaValue::from_serialize(find_symbol_locations(
            symbols_for_symbol.as_ref(),
            &name,
            &symbol_uri,
        ))
    });
    let symbols_for_refs = Arc::clone(&symbols);
    env.add_function("find_references", move |name: String| {
        JinjaValue::from_serialize(find_reference_locations(
            symbols_for_refs.as_ref(),
            &name,
            &reference_uri,
        ))
    });
    if env.add_template("hover", &template).is_err() {
        return Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String(format!(
                "Failed to load template for `{doc_name}`"
            ))),
            range: None,
        });
    }

    let ctx = json!({
        "doc": {
            "name": doc_name,
            "path": template_path,
        },
        "symbol_name": doc_name,
        "hir": hir_value,
    });

    let rendered = match env
        .get_template("hover")
        .and_then(|template| template.render(ctx))
    {
        Ok(rendered) => rendered,
        Err(err) => format!("Failed to render hover template: {err}"),
    };

    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: rendered,
        }),
        range: None,
    })
}

fn hover_template_at_position(
    text: &str,
    rope: &Rope,
    position: Position,
) -> Option<(String, String)> {
    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_idl::language()).ok()?;
    let tree = parser.parse(text, None)?;
    let query = Query::new(&tree_sitter_idl::language(), HOVER_QUERY).ok()?;
    let capture_names = query.capture_names();
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), text.as_bytes());

    while let Some(m) = matches.next() {
        for capture in m.captures {
            let capture_name = match capture_names.get(capture.index as usize) {
                Some(name) => *name,
                None => continue,
            };
            let range = node_range(capture.node, rope);
            if !position_in_range(position, range) {
                continue;
            }
            let name = capture_name.trim_start_matches('@').to_string();
            if name.is_empty() {
                continue;
            }
            if !http::is_http_annotation(&name) {
                continue;
            }
            let template_path = format!("{name}.md");
            return Some((name, template_path));
        }
    }
    None
}

pub(super) fn load_hover_template(path: &str) -> Option<String> {
    let path = path.trim_start_matches('/');
    let asset = HoverDocs::get(path)?;
    let data = asset.data;
    String::from_utf8(data.to_vec()).ok()
}

fn find_symbol_locations(symbols: &[GotoSymbol], name: &str, uri: &str) -> Vec<serde_json::Value> {
    symbols
        .iter()
        .filter(|symbol| symbol.name == name)
        .map(|symbol| {
            let kind = match symbol.kind {
                GotoSymbolKind::Definition => "definition",
                GotoSymbolKind::Declaration => "declaration",
            };
            json!({
                "kind": kind,
                "uri": uri,
                "line": symbol.selection_range.start.line,
                "column": symbol.selection_range.start.character,
                "character": symbol.selection_range.start.character,
            })
        })
        .collect()
}

fn find_reference_locations(
    symbols: &[GotoSymbol],
    name: &str,
    uri: &str,
) -> Vec<serde_json::Value> {
    symbols
        .iter()
        .filter(|symbol| symbol.kind == GotoSymbolKind::Declaration && symbol.name == name)
        .map(|symbol| {
            json!({
                "uri": uri,
                "line": symbol.selection_range.start.line,
                "column": symbol.selection_range.start.character,
                "character": symbol.selection_range.start.character,
            })
        })
        .collect()
}
