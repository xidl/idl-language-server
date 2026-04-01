use log::debug;
use ropey::Rope;
use std::collections::HashSet;
use strsim::jaro_winkler;
use tower_lsp::lsp_types::*;
use tree_sitter::{Node, Parser, Query, QueryCursor, StreamingIterator};
use tree_sitter_highlight::{HighlightConfiguration, HighlightEvent, Highlighter};

use crate::constants::{
    DIAGNOSTIC_SOURCE, DIAGNOSTICS_QUERY, DOCUMENT_SYMBOL_QUERY, FOLDING_QUERY, GOTO_QUERY,
    HIGHLIGHT_NAMES, capture_to_semantic_token,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(crate) struct NodeKey {
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

pub(crate) fn build_document_symbols(text: &str, rope: &Rope) -> Vec<DocumentSymbol> {
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

pub(crate) fn build_diagnostics(text: &str) -> Vec<Diagnostic> {
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
    let mut defined_types = HashSet::new();
    let mut missing_type_diagnostics = Vec::new();
    let mut seen_missing_ranges = HashSet::new();
    let mut captures = cursor.captures(&query, tree.root_node(), text.as_bytes());
    while let Some((m, idx)) = captures.next() {
        let capture = m.captures[*idx];
        let capture_name = match capture_names.get(capture.index as usize) {
            Some(name) => *name,
            None => continue,
        };
        match capture_name {
            "error" => {
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
                    source: Some(DIAGNOSTIC_SOURCE.to_string()),
                    message: "Parse error".to_string(),
                    related_information: None,
                    tags: None,
                    data: None,
                });
            }
            "type.def" => {
                let Some(name) = node_text(capture.node, text) else {
                    continue;
                };
                if !name.is_empty() {
                    defined_types.insert(name);
                }
            }
            "type.ref" => {
                let Some(name) = node_text(capture.node, text) else {
                    continue;
                };
                if name.is_empty() || is_builtin_type(&name) || defined_types.contains(&name) {
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
                let range_key = (
                    range.start.line,
                    range.start.character,
                    range.end.line,
                    range.end.character,
                );
                if !seen_missing_ranges.insert(range_key) {
                    continue;
                }

                missing_type_diagnostics.push((name, range));
            }
            _ => {}
        }
    }

    for (name, range) in missing_type_diagnostics {
        if defined_types.contains(&name) {
            continue;
        }
        let message = match closest_type_match(&name, &defined_types) {
            Some(suggestion) => {
                format!("Unknown type in current file: {name}. Did you mean {suggestion}?")
            }
            None => format!("Unknown type in current file: {name}"),
        };
        diagnostics.push(Diagnostic {
            range,
            severity: Some(DiagnosticSeverity::WARNING),
            code: None,
            code_description: None,
            source: Some(DIAGNOSTIC_SOURCE.to_string()),
            message,
            related_information: None,
            tags: None,
            data: None,
        });
    }
    diagnostics
}

pub(crate) fn build_folding_ranges(text: &str, rope: &Rope) -> Vec<FoldingRange> {
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
pub(crate) enum GotoSymbolKind {
    Definition,
    Declaration,
}

#[derive(Clone, Debug)]
pub(crate) struct GotoSymbol {
    pub(crate) name: String,
    pub(crate) kind: GotoSymbolKind,
    pub(crate) selection_range: Range,
}

pub(crate) fn build_goto_symbols(text: &str, rope: &Rope) -> Vec<GotoSymbol> {
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
        let mut name_node: Option<Node<'_>> = None;

        for capture in m.captures {
            let capture_name = match capture_names.get(capture.index as usize) {
                Some(name) => *name,
                None => continue,
            };
            match capture_name {
                "def" => {
                    symbol_kind = Some(GotoSymbolKind::Definition);
                }
                "def.name" => name_node = Some(capture.node),
                "decl" => {
                    symbol_kind = Some(GotoSymbolKind::Declaration);
                }
                "decl.name" => name_node = Some(capture.node),
                _ => {}
            }
        }

        let (symbol_kind, name_node) = match (symbol_kind, name_node) {
            (Some(kind), Some(name_node)) => (kind, name_node),
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
            selection_range: node_range(name_node, rope),
        });
    }

    symbols
}

pub(crate) fn goto_definition_locations(
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

pub(crate) fn goto_declaration_locations(
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

pub(crate) fn reference_locations(
    symbols: &[GotoSymbol],
    uri: &Url,
    position: Position,
) -> Vec<Location> {
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

pub(crate) fn rename_workspace_edit(
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

fn symbol_at_position(symbols: &[GotoSymbol], position: Position) -> Option<&GotoSymbol> {
    symbols
        .iter()
        .find(|symbol| position_in_range(position, symbol.selection_range))
}

pub(crate) fn position_in_range(position: Position, range: Range) -> bool {
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

#[allow(deprecated)]
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

fn node_text(node: Node<'_>, text: &str) -> Option<String> {
    node.utf8_text(text.as_bytes())
        .ok()
        .map(|name| name.trim().to_string())
}

fn closest_type_match<'a>(name: &str, defined_types: &'a HashSet<String>) -> Option<&'a str> {
    let mut best_match: Option<(&str, f64)> = None;
    let lowercase_name = name.to_ascii_lowercase();
    for candidate in builtin_types()
        .iter()
        .copied()
        .chain(defined_types.iter().map(String::as_str))
    {
        if candidate == name {
            return Some(candidate);
        }
        let lowercase_candidate = candidate.to_ascii_lowercase();
        let score = jaro_winkler(&lowercase_name, &lowercase_candidate);
        if score < 0.80 {
            continue;
        }
        if best_match.is_none_or(|(_, best_score)| score > best_score) {
            best_match = Some((candidate, score));
        }
    }
    best_match.map(|(candidate, _)| candidate)
}

fn builtin_types() -> &'static [&'static str] {
    &[
        "short",
        "int16",
        "long",
        "int32",
        "long long",
        "int64",
        "uint8",
        "boolean",
        "fixed",
        "octet",
        "int8",
        "unsigned short",
        "uint16",
        "unsigned long",
        "uint32",
        "unsigned long long",
        "uint64",
        "float",
        "double",
        "long double",
        "char",
        "wchar",
        "string",
        "wstring",
        "any",
        "Object",
        "ValueBase",
        "sequence",
        "map",
    ]
}

fn is_builtin_type(name: &str) -> bool {
    builtin_types().contains(&name)
}

pub(crate) fn node_range(node: Node<'_>, rope: &Rope) -> Range {
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

pub(crate) fn build_highlight_tokens(text: &str, rope: &Rope) -> Vec<SemanticToken> {
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

    let mut incomplete_tokens: Vec<(u32, u32, u32, u32, u32)> = Vec::new();
    for (start, end, highlight_index) in raw_spans {
        let capture_name = match HIGHLIGHT_NAMES.get(highlight_index) {
            Some(name) => *name,
            None => continue,
        };
        let token = match capture_to_semantic_token(capture_name) {
            Some(token) => token,
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
                    token.token_type,
                    token.token_modifiers_bitset,
                ));
            }
            cur = seg_end;
        }
    }

    incomplete_tokens.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

    let mut tokens = Vec::with_capacity(incomplete_tokens.len());
    let mut pre_line: u32 = 0;
    let mut pre_start: u32 = 0;

    for (line, start, length, token_type, token_modifiers_bitset) in incomplete_tokens {
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
            token_modifiers_bitset,
        });
        pre_line = line;
        pre_start = start;
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc;
    use ropey::Rope;
    use serde_json::json;

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
    fn diagnostics_warns_for_missing_non_builtin_type_in_current_file() {
        let source = r#"
interface Service {
    void takeFoo(Foo value);
};
"#;

        let diagnostics = build_diagnostics(source);
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == Some(DiagnosticSeverity::WARNING)
                && diagnostic.message == "Unknown type in current file: Foo"
        }));
    }

    #[test]
    fn diagnostics_does_not_warn_for_defined_or_builtin_types() {
        let source = r#"
typedef long Foo;

interface Service {
    void takeFoo(Foo value);
    void takeLong(long value);
};
"#;

        let diagnostics = build_diagnostics(source);
        assert!(
            !diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.severity == Some(DiagnosticSeverity::WARNING) })
        );
    }

    #[test]
    fn diagnostics_suggests_builtin_type_name() {
        let source = r#"
interface Service {
    void takeString(strng value);
};
"#;

        let diagnostics = build_diagnostics(source);
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == Some(DiagnosticSeverity::WARNING)
                && diagnostic.message == "Unknown type in current file: strng. Did you mean string?"
        }));
    }

    #[test]
    fn diagnostics_suggests_custom_type_name() {
        let source = r#"
typedef long Foo;

interface Service {
    void takeFoo(Fob value);
};
"#;

        let diagnostics = build_diagnostics(source);
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == Some(DiagnosticSeverity::WARNING)
                && diagnostic.message == "Unknown type in current file: Fob. Did you mean Foo?"
        }));
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
        assert!(rendered.contains("XIDL HTTP mapping"));
    }
}
