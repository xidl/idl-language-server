use std::sync::Arc;

use minijinja::Environment as JinjaEnvironment;
use minijinja::value::Value as JinjaValue;
use ropey::Rope;
use rust_embed::RustEmbed;
use serde_json::json;
use tower_lsp::lsp_types::{
    Hover, HoverContents, MarkedString, MarkupContent, MarkupKind, Position, Url,
};
use tree_sitter::{Node, Parser, Query, QueryCursor, StreamingIterator};
use xidl_parser::parser::parser_text;

use crate::analysis::{
    GotoSymbol, GotoSymbolKind, build_goto_symbols, node_range, position_in_range,
};

pub(crate) mod http;

const HOVER_QUERY: &str = include_str!("../../queries/hover_docs.scm");
const HOVER_TYPES_QUERY: &str = include_str!("../../queries/hover_types.scm");

/// Documentation for builtin types. The canonical names and descriptions come
/// from the xidl language docs; the remaining entries are legacy CORBA
/// spellings that the grammar still accepts, mapped onto the same meaning.
const BUILTIN_DOCS: &[(&str, &str)] = &[
    ("uint8", "The 8-bit unsigned integer type."),
    ("uint16", "The 16-bit unsigned integer type."),
    ("uint32", "The 32-bit unsigned integer type."),
    ("uint64", "The 64-bit unsigned integer type."),
    ("int8", "The 8-bit signed integer type."),
    ("int16", "The 16-bit signed integer type."),
    ("int32", "The 32-bit signed integer type."),
    ("int64", "The 64-bit signed integer type."),
    (
        "float32",
        "The 32-bit single-precision floating-point type.",
    ),
    (
        "float64",
        "The 64-bit double-precision floating-point type.",
    ),
    ("boolean", "The boolean type."),
    ("string", "The string type."),
    ("sequence", "Variable-length array with elements of type T."),
    (
        "map",
        "Key-value pair collection with keys of type K and values of type V.",
    ),
    // Legacy CORBA spellings accepted by the grammar.
    ("short", "The 16-bit signed integer type."),
    ("long", "The 32-bit signed integer type."),
    ("long long", "The 64-bit signed integer type."),
    ("unsigned short", "The 16-bit unsigned integer type."),
    ("unsigned long", "The 32-bit unsigned integer type."),
    ("unsigned long long", "The 64-bit unsigned integer type."),
    ("float", "The 32-bit single-precision floating-point type."),
    ("double", "The 64-bit double-precision floating-point type."),
    // Additional builtin spellings recognized by the grammar.
    ("octet", "The octet type (an 8-bit byte)."),
    ("char", "The character type."),
    ("wchar", "The wide character type."),
    ("wstring", "The wide string type."),
    ("any", "The any type, which can hold a value of any type."),
    ("Object", "The object reference type."),
    ("ValueBase", "The base type for value types."),
    ("fixed", "The fixed-point type."),
];

#[derive(RustEmbed)]
#[folder = "docs/hover"]
struct HoverDocs;

pub(super) fn build_hover(text: &str, rope: &Rope, uri: &Url, position: Position) -> Option<Hover> {
    if let Some((doc_name, template_path)) = hover_template_at_position(text, rope, position) {
        return Some(annotation_hover(text, rope, uri, &doc_name, &template_path));
    }
    hover_type_at_position(text, rope, position)
}

fn annotation_hover(
    text: &str,
    rope: &Rope,
    uri: &Url,
    doc_name: &str,
    template_path: &str,
) -> Hover {
    let template = match load_hover_template(template_path) {
        Some(template) => template,
        None => {
            return Hover {
                contents: HoverContents::Scalar(MarkedString::String(format!(
                    "No documentation template found for `{doc_name}`"
                ))),
                range: None,
            };
        }
    };

    let symbols = Arc::new(build_goto_symbols(text, rope));
    let hir_value = match parser_text(text) {
        Ok(spec) => {
            let hir = xidl_parser::hir::Specification::from(spec);
            serde_json::to_value(hir).unwrap_or(serde_json::Value::Null)
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
        return Hover {
            contents: HoverContents::Scalar(MarkedString::String(format!(
                "Failed to load template for `{doc_name}`"
            ))),
            range: None,
        };
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

    Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: rendered,
        }),
        range: None,
    }
}

pub(crate) fn build_inspect_value(text: &str, target: InspectTarget) -> serde_json::Value {
    match parser_text(text) {
        Ok(spec) => match target {
            InspectTarget::Hir => {
                let hir = xidl_parser::hir::Specification::from(spec);
                serde_json::to_value(hir).unwrap_or(serde_json::Value::Null)
            }
            InspectTarget::TypedAst => {
                serde_json::to_value(spec).unwrap_or(serde_json::Value::Null)
            }
        },
        Err(_) => serde_json::Value::Null,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InspectTarget {
    Hir,
    TypedAst,
}

fn hover_type_at_position(text: &str, rope: &Rope, position: Position) -> Option<Hover> {
    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_idl::language()).ok()?;
    let tree = parser.parse(text, None)?;
    let query = Query::new(&tree_sitter_idl::language(), HOVER_TYPES_QUERY).ok()?;
    let capture_names = query.capture_names();

    let mut builtins: Vec<Node<'_>> = Vec::new();
    let mut defs: Vec<(Node<'_>, Node<'_>)> = Vec::new();
    let mut type_refs: Vec<Node<'_>> = Vec::new();

    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), text.as_bytes());
    while let Some(m) = matches.next() {
        let mut def = None;
        let mut def_name = None;
        for capture in m.captures {
            let capture_name = match capture_names.get(capture.index as usize) {
                Some(name) => *name,
                None => continue,
            };
            match capture_name {
                "def" => def = Some(capture.node),
                "def.name" => def_name = Some(capture.node),
                "type.name" => type_refs.push(capture.node),
                "builtin.type" => builtins.push(capture.node),
                _ => {}
            }
        }
        if let (Some(def), Some(def_name)) = (def, def_name) {
            defs.push((def, def_name));
        }
    }

    let contains = |node: Node<'_>| position_in_range(position, node_range(node, rope));

    // Prefer the most specific node under the cursor, so hovering the element
    // type inside `sequence<int32>` shows the element docs, not the container.
    if let Some(node) = builtins
        .iter()
        .copied()
        .filter(|node| contains(*node))
        .min_by_key(|node| node.byte_range().len())
    {
        return builtin_hover(text, node);
    }

    // Hovering a type's own declaration name shows the whole definition.
    if let Some((def, _)) = defs.iter().copied().find(|(_, name)| contains(*name)) {
        return Some(definition_hover(text, def));
    }

    // Hovering a type reference resolves it back to its definition.
    if let Some(node) = type_refs
        .iter()
        .copied()
        .filter(|node| contains(*node))
        .min_by_key(|node| node.byte_range().len())
    {
        let name = match node.utf8_text(text.as_bytes()) {
            Ok(name) => name.trim(),
            Err(_) => return None,
        };
        if let Some((def, _)) = defs
            .iter()
            .filter(|(_, name_node)| {
                name_node
                    .utf8_text(text.as_bytes())
                    .map(|candidate| candidate.trim() == name)
                    .unwrap_or(false)
            })
            .max_by_key(|(def, _)| def.byte_range().len())
        {
            return Some(definition_hover(text, *def));
        }
        // Spellings such as `float32`/`float64` parse as type references but
        // are still documented builtins in xidl.
        return builtin_named_hover(name);
    }

    None
}

fn definition_hover(text: &str, def: Node<'_>) -> Hover {
    let source = def
        .utf8_text(text.as_bytes())
        .map(|source| source.trim())
        .unwrap_or("");
    Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: format!("```idl\n{source}\n```"),
        }),
        range: None,
    }
}

fn builtin_hover(text: &str, node: Node<'_>) -> Option<Hover> {
    let raw = node.utf8_text(text.as_bytes()).ok()?;
    let key = builtin_key(raw);
    let (_, doc) = BUILTIN_DOCS.iter().find(|(name, _)| *name == key)?;
    Some(builtin_hover_content(raw.trim(), doc))
}

fn builtin_named_hover(name: &str) -> Option<Hover> {
    let (_, doc) = BUILTIN_DOCS
        .iter()
        .find(|(candidate, _)| *candidate == name)?;
    Some(builtin_hover_content(name, doc))
}

fn builtin_hover_content(name: &str, doc: &str) -> Hover {
    Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: format!("```idl\n{name}\n```\n\n{doc}"),
        }),
        range: None,
    }
}

/// Map a builtin node's source text onto a `BUILTIN_DOCS` key. Container and
/// bounded spellings (`sequence<int32>`, `string<32>`) only keep the keyword,
/// while multi-word spellings (`long long`) match exactly.
fn builtin_key(raw: &str) -> &str {
    if BUILTIN_DOCS.iter().any(|(name, _)| *name == raw) {
        raw
    } else {
        raw.split(|c: char| c == '<' || c.is_whitespace())
            .next()
            .unwrap_or(raw)
    }
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
            if capture_name != "annotation" {
                continue;
            }
            let range = node_range(capture.node, rope);
            if !position_in_range(position, range) {
                continue;
            }
            let raw = capture.node.utf8_text(text.as_bytes()).ok()?.trim();
            let name = raw
                .trim_start_matches('@')
                .split(|character: char| character == '(' || character.is_whitespace())
                .next()
                .unwrap_or("")
                .to_ascii_lowercase();
            if name.is_empty() {
                continue;
            }
            if !http::is_http_annotation(&name) {
                continue;
            }
            let template_path = format!("{name}.md");
            if load_hover_template(&template_path).is_none() {
                continue;
            }
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

#[cfg(test)]
mod tests {
    use super::{build_hover, hover_template_at_position, hover_type_at_position};
    use ropey::Rope;
    use tower_lsp::lsp_types::{HoverContents, Position, Url};

    fn hover_markdown(text: &str, position: Position) -> String {
        let rope = Rope::from_str(text);
        let hover = hover_type_at_position(text, &rope, position)
            .expect("expected a hover at the given position");
        match hover.contents {
            HoverContents::Markup(content) => content.value,
            HoverContents::Scalar(_) | HoverContents::Array(_) => {
                panic!("expected markup hover contents")
            }
        }
    }

    #[test]
    fn hover_template_matches_get_annotation() {
        let text = "@get(path = \"/users\")\ninterface UserService { any users(); };";
        let rope = Rope::from_str(text);

        assert_eq!(
            hover_template_at_position(text, &rope, Position::new(0, 2)),
            Some(("get".to_string(), "get.md".to_string()))
        );
    }

    #[test]
    fn hover_type_reference_shows_definition() {
        let text = "struct SimulationRequest {\n    string src_ip;\n    string dst_ip;\n    string proto;\n    int32 port;\n};\n\nstruct Other {\n    SimulationRequest simulation_req;\n};";
        let value = hover_markdown(text, Position::new(8, 6));

        assert!(
            value.contains("```idl"),
            "expected a code fence in: {value}"
        );
        assert!(
            value.contains("struct SimulationRequest {"),
            "missing definition in: {value}"
        );
        assert!(
            value.contains("int32 port;"),
            "missing definition body in: {value}"
        );
    }

    #[test]
    fn hover_type_definition_name_shows_definition() {
        let text = "struct SimulationRequest {\n    string src_ip;\n    int32 port;\n};";
        let offset = text.find("SimulationRequest").unwrap() as u32;
        let value = hover_markdown(text, Position::new(0, offset + 2));

        assert!(
            value.contains("struct SimulationRequest {"),
            "missing definition in: {value}"
        );
        assert!(
            value.contains("```idl"),
            "expected a code fence in: {value}"
        );
    }

    #[test]
    fn hover_typedef_reference_shows_definition() {
        let text = "typedef sequence<int32> IntList;\nstruct A { IntList list; };";
        let value = hover_markdown(text, Position::new(1, 13));

        assert!(
            value.contains("typedef sequence<int32> IntList"),
            "missing typedef in: {value}"
        );
    }

    #[test]
    fn hover_builtin_int32_shows_doc() {
        let text = "struct S { int32 port; };";
        let offset = text.find("int32").unwrap() as u32;
        let value = hover_markdown(text, Position::new(0, offset + 2));

        assert!(
            value.contains("The 32-bit signed integer type."),
            "got: {value}"
        );
        assert!(
            value.contains("```idl"),
            "expected a code fence in: {value}"
        );
    }

    #[test]
    fn hover_builtin_sequence_keyword_shows_doc() {
        let text = "struct S { sequence<int32> values; };";
        let offset = text.find("sequence").unwrap() as u32;
        let value = hover_markdown(text, Position::new(0, offset + 2));

        assert!(value.contains("Variable-length array"), "got: {value}");
    }

    #[test]
    fn hover_builtin_map_keyword_shows_doc() {
        let text = "struct S { map<string, int32> table; };";
        let offset = text.find("map").unwrap() as u32;
        let value = hover_markdown(text, Position::new(0, offset + 2));

        assert!(value.contains("Key-value pair collection"), "got: {value}");
    }

    #[test]
    fn hover_builtin_long_long_shows_doc() {
        let text = "interface I { void f(long long value); };";
        let offset = text.find("long long").unwrap() as u32;
        let value = hover_markdown(text, Position::new(0, offset + 2));

        assert!(
            value.contains("The 64-bit signed integer type."),
            "got: {value}"
        );
    }

    #[test]
    fn hover_builtin_string_keyword_shows_doc() {
        let text = "struct S { string name; };";
        let offset = text.find("string").unwrap() as u32;
        let value = hover_markdown(text, Position::new(0, offset + 2));

        assert!(value.contains("The string type."), "got: {value}");
    }

    #[test]
    fn hover_float32_reference_falls_back_to_builtin_doc() {
        let text = "interface I { void f(float32 value); };";
        let offset = text.find("float32").unwrap() as u32;
        let value = hover_markdown(text, Position::new(0, offset + 2));

        assert!(
            value.contains("The 32-bit single-precision floating-point type."),
            "got: {value}"
        );
    }

    #[test]
    fn build_hover_routes_to_type_definition() {
        let text = "struct SimulationRequest {\n    int32 port;\n};\n\nstruct Other { SimulationRequest simulation_req; };";
        let rope = Rope::from_str(text);
        let uri = Url::parse("file:///test.idl").unwrap();

        let hover = build_hover(text, &rope, &uri, Position::new(4, 17)).expect("hover");
        match hover.contents {
            HoverContents::Markup(content) => {
                assert!(
                    content.value.contains("struct SimulationRequest {"),
                    "got: {}",
                    content.value
                );
            }
            _ => panic!("expected markup hover contents"),
        }
    }

    #[test]
    fn hover_reference_prefers_full_definition_over_forward_decl() {
        let text = "struct SimulationRequest;\nstruct Other { SimulationRequest simulation_req; };\nstruct SimulationRequest { string src_ip; int32 port; };";
        let value = hover_markdown(text, Position::new(1, 17));

        assert!(
            value.contains("int32 port;"),
            "expected the full definition, got: {value}"
        );
    }

    #[test]
    fn hover_within_sequence_element_shows_element_doc() {
        let text = "struct S { sequence<int32> values; };";
        let offset = text.find("int32").unwrap() as u32;
        let value = hover_markdown(text, Position::new(0, offset + 2));

        assert!(
            value.contains("The 32-bit signed integer type."),
            "got: {value}"
        );
    }
}
