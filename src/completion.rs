use std::collections::HashSet;

use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse,
};

use crate::analysis::{GotoSymbolKind, build_goto_symbols, position_to_byte};
use crate::context::AppContext;

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
    let offset = position_to_byte(&rope, params.text_document_position.position);
    let prefix_start = text[..offset]
        .char_indices()
        .rev()
        .find_map(|(index, character)| {
            (!character.is_ascii_alphanumeric() && character != '_')
                .then_some(index + character.len_utf8())
        })
        .unwrap_or(0);
    if prefix_start > 0 && text.as_bytes()[prefix_start - 1] == b'@' {
        return None;
    }

    let prefix = &text[prefix_start..offset];
    let mut items = Vec::new();
    let mut seen = HashSet::new();

    for keyword in KEYWORDS {
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
