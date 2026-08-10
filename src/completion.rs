use std::collections::HashSet;

use ropey::Rope;
use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse, CompletionTextEdit,
    InsertTextFormat, Position, Range, TextEdit,
};

use crate::analysis::{GotoSymbolKind, build_goto_symbols, byte_to_position, position_to_byte};
use crate::context::AppContext;
use crate::snippets::{self, Snippet};

const KEYWORDS: &[&str] = &[
    "module",
    "interface",
    "struct",
    "union",
    "enum",
    "bitmask",
    "bitset",
    "typedef",
    "const",
    "exception",
    "annotation",
    "import",
    "readonly",
    "attribute",
    "void",
    "in",
    "out",
    "inout",
    "switch",
    "case",
    "default",
    "raises",
];

pub(crate) fn completion(ctx: &AppContext, params: CompletionParams) -> Option<CompletionResponse> {
    let uri = params.text_document_position.text_document.uri;
    let rope = ctx.document_map.get(uri.as_str())?;
    let text = rope.to_string();
    let position = params.text_document_position.position;
    let offset = position_to_byte(&rope, position);
    let prefix_start = text[..offset]
        .char_indices()
        .rev()
        .find_map(|(index, character)| {
            (!character.is_ascii_alphanumeric() && character != '_')
                .then_some(index + character.len_utf8())
        })
        .unwrap_or(0);
    // Snippets that start with `@` or `#` only complete in their own context,
    // and the replace range must include the leading character.
    let special = if prefix_start > 0 {
        text.as_bytes()[prefix_start - 1]
    } else {
        0
    };
    let prefix = &text[prefix_start..offset];

    let mut items = Vec::new();
    let mut seen = HashSet::new();

    if special != 0 {
        if ctx.snippet_support() {
            add_snippet_items(
                &mut items,
                &mut seen,
                &rope,
                prefix_start - 1,
                position,
                prefix,
                special,
            );
        }
    } else {
        if ctx.snippet_support() {
            add_snippet_items(
                &mut items,
                &mut seen,
                &rope,
                prefix_start,
                position,
                prefix,
                0,
            );
        }
        for keyword in KEYWORDS {
            if seen.contains(*keyword) {
                continue;
            }
            add_item(
                &mut items,
                &mut seen,
                (*keyword).to_string(),
                CompletionItemKind::KEYWORD,
                "IDL keyword",
                prefix,
            );
        }
        for builtin in crate::analysis::builtin_types() {
            add_item(
                &mut items,
                &mut seen,
                (*builtin).to_string(),
                CompletionItemKind::TYPE_PARAMETER,
                "IDL built-in type",
                prefix,
            );
        }
        for symbol in build_goto_symbols(&text, &rope) {
            if symbol.kind == GotoSymbolKind::Definition {
                add_item(
                    &mut items,
                    &mut seen,
                    symbol.name,
                    CompletionItemKind::CLASS,
                    "Type or interface in this file",
                    prefix,
                );
            }
        }
    }

    items.sort_by(|left, right| left.label.cmp(&right.label));
    Some(CompletionResponse::Array(items))
}

fn add_item(
    items: &mut Vec<CompletionItem>,
    seen: &mut HashSet<String>,
    label: String,
    kind: CompletionItemKind,
    detail: &str,
    prefix: &str,
) {
    if !label.starts_with(prefix) || !seen.insert(label.clone()) {
        return;
    }

    let mut item = CompletionItem::new_simple(label, detail.to_string());
    item.kind = Some(kind);
    items.push(item);
}

fn add_snippet_items(
    items: &mut Vec<CompletionItem>,
    seen: &mut HashSet<String>,
    rope: &Rope,
    replace_start_byte: usize,
    cursor: Position,
    prefix: &str,
    special: u8,
) {
    let replace_start = byte_to_position(rope, replace_start_byte);
    for snippet in snippets::all() {
        let Some(label) = matching_label(snippet, special, prefix) else {
            continue;
        };
        if !seen.insert(label.to_string()) {
            continue;
        }
        items.push(snippet_item(snippet, label, replace_start, cursor));
    }
}

/// Returns the label (a snippet prefix) when `snippet` matches the current
/// completion context, or `None` otherwise.
///
/// `special` is `b'@'`, `b'#'` or `0` for normal text. Snippets prefixed with
/// `@`/`#` only match inside their own special context, and are matched against
/// the last whitespace-separated token so that e.g. `#pragma version` matches
/// when the user has typed `v` after `#` or `#pragma `.
fn matching_label<'a>(snippet: &'a Snippet, special: u8, prefix: &str) -> Option<&'a str> {
    for candidate in &snippet.prefixes {
        let leading = *candidate.as_bytes().first()?;
        let is_special = leading == b'@' || leading == b'#';
        if special == 0 {
            if is_special {
                continue;
            }
        } else if leading != special {
            continue;
        }
        let key = if is_special {
            candidate[1..].split_whitespace().last().unwrap_or("")
        } else {
            candidate.as_str()
        };
        if key.starts_with(prefix) {
            return Some(candidate);
        }
    }
    None
}

fn snippet_item(
    snippet: &Snippet,
    label: &str,
    replace_start: Position,
    cursor: Position,
) -> CompletionItem {
    CompletionItem {
        label: label.to_string(),
        kind: Some(CompletionItemKind::SNIPPET),
        detail: snippet.description.clone(),
        insert_text: Some(snippet.body.clone()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        text_edit: Some(CompletionTextEdit::Edit(TextEdit {
            range: Range {
                start: replace_start,
                end: cursor,
            },
            new_text: snippet.body.clone(),
        })),
        ..CompletionItem::default()
    }
}

#[cfg(test)]
mod tests {
    use super::matching_label;

    fn snippet(prefixes: &[&str]) -> crate::snippets::Snippet {
        crate::snippets::Snippet {
            prefixes: prefixes.iter().map(|prefix| prefix.to_string()).collect(),
            body: "body".to_string(),
            description: None,
        }
    }

    #[test]
    fn normal_context_matches_plain_prefixes() {
        let s = snippet(&["module"]);
        assert_eq!(matching_label(&s, 0, "mod"), Some("module"));
        assert_eq!(matching_label(&s, 0, "x"), None);
        assert_eq!(matching_label(&s, b'@', "mod"), None);
    }

    #[test]
    fn annotation_context_matches_at_snippets() {
        let s = snippet(&["@get"]);
        assert_eq!(matching_label(&s, b'@', "g"), Some("@get"));
        assert_eq!(matching_label(&s, b'@', ""), Some("@get"));
        assert_eq!(matching_label(&s, b'@', "x"), None);
        assert_eq!(matching_label(&s, 0, "get"), None);
    }

    #[test]
    fn pragma_context_matches_last_token() {
        let s = snippet(&["#pragma version"]);
        assert_eq!(matching_label(&s, b'#', ""), Some("#pragma version"));
        assert_eq!(matching_label(&s, b'#', "v"), Some("#pragma version"));
        assert_eq!(matching_label(&s, b'#', "ver"), Some("#pragma version"));
        assert_eq!(matching_label(&s, b'#', "p"), None);
        assert_eq!(matching_label(&s, b'#', "x"), None);
        assert_eq!(matching_label(&s, 0, "version"), None);
    }
}
