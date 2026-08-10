use std::collections::HashSet;

use ropey::Rope;
use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse, CompletionTextEdit,
    InsertTextFormat, Position, Range, TextEdit,
};

use crate::analysis::{
    builtin_annotations, builtin_pragmas, builtin_types, byte_to_position,
    collect_completion_symbols, position_to_byte,
};
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

/// Keywords that can legally appear where a type is expected (interface bodies,
/// union case arms), so focused type completion does not hide them.
const TYPE_KEYWORDS: &[&str] = &["void", "attribute", "readonly", "case", "default"];

/// Where the cursor is, which decides which completion items are offered.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CompletionContext {
    /// Right after `@`: builtin and custom annotations.
    Annotation,
    /// On a `#pragma` line: builtin pragma forms.
    Pragma,
    /// In a type position: focused list of builtin and custom types.
    Type,
    /// Anything else: keywords, types and plain snippets.
    Normal,
}

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
    let prefix = &text[prefix_start..offset];

    let context = completion_context(&text[..offset], prefix_start);
    // Snippets that start with `@` or `#` only complete in their own context,
    // and the replace range must include the leading character. Pragma items
    // replace from the `#` even when whitespace follows (e.g. `#pragma xidlc p`).
    let (replace_start_byte, key) = match context {
        CompletionContext::Annotation => (prefix_start.saturating_sub(1), prefix),
        CompletionContext::Pragma => (pragma_start_byte(&text, prefix_start), prefix),
        CompletionContext::Type | CompletionContext::Normal => (prefix_start, prefix),
    };
    let replace_start = byte_to_position(&rope, replace_start_byte);
    let site = CompletionSite {
        replace_start,
        cursor: position,
        prefix: key,
    };

    let mut items = Vec::new();
    let mut seen = HashSet::new();

    match context {
        CompletionContext::Annotation => {
            if ctx.snippet_support() {
                add_snippet_items(
                    &mut items,
                    &mut seen,
                    &rope,
                    replace_start_byte,
                    position,
                    key,
                    b'@',
                );
            }
            for name in builtin_annotations() {
                add_annotation_item(
                    &mut items,
                    &mut seen,
                    &site,
                    name,
                    "IDL built-in annotation",
                    CompletionItemKind::PROPERTY,
                );
            }
            for name in collect_completion_symbols(&text).annotations {
                add_annotation_item(
                    &mut items,
                    &mut seen,
                    &site,
                    &name,
                    "Custom annotation in this file",
                    CompletionItemKind::FUNCTION,
                );
            }
        }
        CompletionContext::Pragma => {
            for (label, body) in builtin_pragmas() {
                add_pragma_item(
                    &mut items,
                    &mut seen,
                    &site,
                    label,
                    body,
                    ctx.snippet_support(),
                );
            }
            if ctx.snippet_support() {
                add_snippet_items(
                    &mut items,
                    &mut seen,
                    &rope,
                    replace_start_byte,
                    position,
                    key,
                    b'#',
                );
            }
        }
        CompletionContext::Type | CompletionContext::Normal => {
            if ctx.snippet_support() {
                add_snippet_items(
                    &mut items,
                    &mut seen,
                    &rope,
                    replace_start_byte,
                    position,
                    key,
                    0,
                );
            }
            for keyword in KEYWORDS {
                if context == CompletionContext::Type && !TYPE_KEYWORDS.contains(keyword) {
                    continue;
                }
                if seen.contains(*keyword) {
                    continue;
                }
                add_item(
                    &mut items,
                    &mut seen,
                    (*keyword).to_string(),
                    CompletionItemKind::KEYWORD,
                    "IDL keyword",
                    key,
                );
            }
            for builtin in builtin_types() {
                add_type_item(&mut items, &mut seen, &site, builtin, "IDL built-in type");
            }
            for name in collect_completion_symbols(&text).types {
                add_type_item(
                    &mut items,
                    &mut seen,
                    &site,
                    &name,
                    "Type or interface in this file",
                );
            }
        }
    }

    items.sort_by(|left, right| left.label.cmp(&right.label));
    Some(CompletionResponse::Array(items))
}

fn completion_context(before: &str, prefix_start: usize) -> CompletionContext {
    // The byte immediately before the typed word. A `@` or `#` there means the
    // user is typing an annotation or a pragma directive.
    let special = if prefix_start > 0 {
        before.as_bytes()[prefix_start - 1]
    } else {
        0
    };
    if special == b'@' {
        return CompletionContext::Annotation;
    }
    if special == b'#' {
        return CompletionContext::Pragma;
    }

    // `#pragma ...` continues past whitespace, so also check the line start.
    let line_start = before[..prefix_start]
        .rfind('\n')
        .map(|i| i + 1)
        .unwrap_or(0);
    if before[line_start..prefix_start]
        .trim_start()
        .starts_with('#')
    {
        return CompletionContext::Pragma;
    }

    if type_position(before, prefix_start) {
        return CompletionContext::Type;
    }
    CompletionContext::Normal
}

/// Byte offset of the `#` that starts the pragma on the current line.
fn pragma_start_byte(text: &str, prefix_start: usize) -> usize {
    let line_start = text[..prefix_start].rfind('\n').map(|i| i + 1).unwrap_or(0);
    text[line_start..prefix_start]
        .find('#')
        .map(|index| line_start + index)
        .unwrap_or(prefix_start)
}

/// The significant tokens (words and single-char punctuation, whitespace
/// dropped) that precede the typed word, most recent first.
fn significant_tokens(s: &str, max: usize) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut word = String::new();
    for ch in s.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            word.push(ch);
        } else {
            if !word.is_empty() {
                tokens.push(std::mem::take(&mut word));
            }
            if !ch.is_whitespace() {
                tokens.push(ch.to_string());
            }
        }
    }
    if !word.is_empty() {
        tokens.push(word);
    }
    tokens.into_iter().rev().take(max).collect()
}

/// Heuristic: is the cursor in a position where a type name is expected?
///
/// tree-sitter-idl frequently reports the whole document as an ERROR node for
/// incomplete input, so instead of relying on the AST we look at the
/// significant tokens before the cursor: after `{`/`;`/`,`/`:`/`<` inside a
/// body, after `typedef`/`const`/`attribute`/`in`/`out`/`inout`/`raises`/
/// `switch`, or after an annotation.
fn type_position(before: &str, prefix_start: usize) -> bool {
    let tokens = significant_tokens(&before[..prefix_start], 12);
    let Some(last) = tokens.first() else {
        return false;
    };
    match last.as_str() {
        "{" => matches!(
            body_keyword(&tokens),
            Some(kind) if matches!(kind.as_str(), "struct" | "union" | "exception" | "annotation" | "interface")
        ),
        ";" => !matches!(tokens.get(1).map(String::as_str), Some("}")),
        "," | ":" | "<" => true,
        "(" => {
            // `(` after `raises`/`switch`/an op name opens a type list, but
            // `@Anno(` opens annotation arguments instead.
            tokens.get(1).is_none() || tokens.get(2).map(String::as_str) != Some("@")
        }
        word if is_type_introducer(word) => true,
        // After an annotation comes the annotated member's type.
        word if tokens.get(1).map(String::as_str) == Some("@") && word != "@" => true,
        _ => false,
    }
}

fn is_type_introducer(word: &str) -> bool {
    matches!(
        word,
        "typedef" | "const" | "attribute" | "in" | "out" | "inout" | "raises" | "switch"
    )
}

/// Nearest construct keyword before an opening `{`, if any.
fn body_keyword(tokens: &[String]) -> Option<String> {
    const BODY_KEYWORDS: &[&str] = &[
        "module",
        "interface",
        "struct",
        "union",
        "enum",
        "bitmask",
        "bitset",
        "exception",
        "annotation",
    ];
    tokens
        .iter()
        .skip(1)
        .take(8)
        .find(|token| BODY_KEYWORDS.contains(&token.as_str()))
        .cloned()
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

/// Shared cursor state for building text-edit completions.
struct CompletionSite<'a> {
    /// Start of the replacement range (includes `@`/`#` for annotations and
    /// pragmas).
    replace_start: Position,
    /// Current cursor position.
    cursor: Position,
    /// Typed prefix being completed.
    prefix: &'a str,
}

/// A type completion replacing the typed prefix, so multi-word labels such as
/// `long long` replace cleanly.
fn add_type_item(
    items: &mut Vec<CompletionItem>,
    seen: &mut HashSet<String>,
    site: &CompletionSite<'_>,
    name: &str,
    detail: &str,
) {
    if !name.starts_with(site.prefix) || !seen.insert(name.to_string()) {
        return;
    }
    items.push(CompletionItem {
        label: name.to_string(),
        kind: Some(CompletionItemKind::CLASS),
        detail: Some(detail.to_string()),
        text_edit: Some(CompletionTextEdit::Edit(TextEdit {
            range: Range {
                start: site.replace_start,
                end: site.cursor,
            },
            new_text: name.to_string(),
        })),
        ..CompletionItem::default()
    });
}

/// An annotation completion: the label includes the leading `@` and the edit
/// replaces from it.
fn add_annotation_item(
    items: &mut Vec<CompletionItem>,
    seen: &mut HashSet<String>,
    site: &CompletionSite<'_>,
    name: &str,
    detail: &str,
    kind: CompletionItemKind,
) {
    let label = format!("@{name}");
    if !label.starts_with(site.prefix) || !seen.insert(label.clone()) {
        return;
    }
    items.push(CompletionItem {
        label,
        kind: Some(kind),
        detail: Some(detail.to_string()),
        text_edit: Some(CompletionTextEdit::Edit(TextEdit {
            range: Range {
                start: site.replace_start,
                end: site.cursor,
            },
            new_text: format!("@{name}"),
        })),
        ..CompletionItem::default()
    });
}

/// A pragma completion. Matches against the last whitespace-separated word so
/// `#pragma xidlc p` and `#p` both match `#pragma xidlc package`, and the edit
/// replaces from the leading `#`.
fn add_pragma_item(
    items: &mut Vec<CompletionItem>,
    seen: &mut HashSet<String>,
    site: &CompletionSite<'_>,
    label: &str,
    snippet_body: &str,
    snippet_support: bool,
) {
    let key = label[1..].split_whitespace().last().unwrap_or("");
    if !key.starts_with(site.prefix) || !seen.insert(label.to_string()) {
        return;
    }
    let (new_text, insert_text_format) = if snippet_support {
        (snippet_body.to_string(), Some(InsertTextFormat::SNIPPET))
    } else {
        (strip_snippet_placeholders(snippet_body), None)
    };
    items.push(CompletionItem {
        label: label.to_string(),
        kind: Some(CompletionItemKind::KEYWORD),
        detail: Some("Built-in XIDL pragma".to_string()),
        insert_text: Some(new_text.clone()),
        insert_text_format,
        text_edit: Some(CompletionTextEdit::Edit(TextEdit {
            range: Range {
                start: site.replace_start,
                end: site.cursor,
            },
            new_text,
        })),
        ..CompletionItem::default()
    });
}

/// Removes `${1:...}`/`$0` placeholders so a snippet body doubles as plain
/// text when the client does not support snippets.
fn strip_snippet_placeholders(body: &str) -> String {
    let mut out = String::with_capacity(body.len());
    let mut chars = body.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '$' {
            if chars.peek() == Some(&'{') {
                // Skip until the matching `}`.
                let mut depth = 0;
                for next in chars.by_ref() {
                    if next == '{' {
                        depth += 1;
                    } else if next == '}' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                }
                continue;
            }
            if matches!(chars.peek(), Some('0'..='9')) {
                chars.next();
                continue;
            }
        }
        out.push(ch);
    }
    out
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
    use super::*;

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

    /// Computes the trailing-word start exactly like `completion()` so tests
    /// only need to pass the text before the cursor.
    fn ctx(text: &str) -> CompletionContext {
        let offset = text.len();
        let prefix_start = text
            .char_indices()
            .rev()
            .find_map(|(index, character)| {
                (!character.is_ascii_alphanumeric() && character != '_')
                    .then_some(index + character.len_utf8())
            })
            .unwrap_or(0);
        completion_context(&text[..offset], prefix_start)
    }

    #[test]
    fn annotation_context_after_at_sign() {
        assert_eq!(ctx("@ke"), CompletionContext::Annotation);
        assert_eq!(ctx("@"), CompletionContext::Annotation);
        assert_eq!(ctx("struct S { @op"), CompletionContext::Annotation);
    }

    #[test]
    fn pragma_context_on_hash_line() {
        assert_eq!(ctx("#p"), CompletionContext::Pragma);
        assert_eq!(ctx("#pragma xidlc p"), CompletionContext::Pragma);
        assert_eq!(ctx("  #pragma "), CompletionContext::Pragma);
        assert_eq!(
            ctx("const long X = 1; // # not pragma"),
            CompletionContext::Normal
        );
    }

    #[test]
    fn type_context_in_bodies() {
        assert_eq!(ctx("struct S { lo"), CompletionContext::Type);
        assert_eq!(ctx("struct S { long x; lo"), CompletionContext::Type);
        assert_eq!(ctx("struct S {\n    lo"), CompletionContext::Type);
        assert_eq!(ctx("typedef lo"), CompletionContext::Type);
        assert_eq!(ctx("const lo"), CompletionContext::Type);
        assert_eq!(ctx("interface I { void op(in lo"), CompletionContext::Type);
        assert_eq!(
            ctx("interface I { void op(in long x, out st"),
            CompletionContext::Type
        );
        assert_eq!(ctx("sequence<lo"), CompletionContext::Type);
        assert_eq!(ctx("map<string, lo"), CompletionContext::Type);
        assert_eq!(ctx("raises (Ex"), CompletionContext::Type);
        assert_eq!(ctx("union U switch (lo"), CompletionContext::Type);
        assert_eq!(ctx("attribute lo"), CompletionContext::Type);
        assert_eq!(ctx("struct S { @key lo"), CompletionContext::Type);
        assert_eq!(ctx("struct S { @MyAnno lo"), CompletionContext::Type);
    }

    #[test]
    fn non_type_positions_stay_normal() {
        assert_eq!(ctx("module M { st"), CompletionContext::Normal);
        assert_eq!(ctx(""), CompletionContext::Normal);
        assert_eq!(ctx("st"), CompletionContext::Normal);
        assert_eq!(
            ctx("interface I { void op(in long x); }\nlo"),
            CompletionContext::Normal
        );
        assert_eq!(ctx("import lo"), CompletionContext::Normal);
        assert_eq!(ctx("struct S { long x; };\nlo"), CompletionContext::Normal);
        assert_eq!(ctx("const long X = lo"), CompletionContext::Normal);
        assert_eq!(ctx("enum E { lo"), CompletionContext::Normal);
        assert_eq!(ctx("@MyAnno(lo"), CompletionContext::Normal);
        assert_eq!(
            ctx("struct S { sequence<long> lo"),
            CompletionContext::Normal
        );
    }

    #[test]
    fn pragma_start_byte_locates_hash() {
        assert_eq!(pragma_start_byte("#p", 1), 0);
        assert_eq!(pragma_start_byte("#pragma xidlc p", 14), 0);
        assert_eq!(pragma_start_byte("  #pragma v", 10), 2);
    }

    #[test]
    fn builtin_annotation_names_are_known() {
        assert!(builtin_annotations().contains(&"key"));
        assert!(builtin_annotations().contains(&"optional"));
        assert!(builtin_annotations().contains(&"range"));
        assert!(builtin_annotations().contains(&"DDSService"));
    }

    #[test]
    fn builtin_pragma_labels_and_bodies() {
        let pragmas = builtin_pragmas();
        assert!(
            pragmas
                .iter()
                .any(|(label, _)| *label == "#pragma xidlc package")
        );
        assert!(
            pragmas
                .iter()
                .any(|(label, _)| *label == "#pragma xidlc openapi service")
        );
        let (label, body) = pragmas
            .iter()
            .find(|(label, _)| *label == "#pragma xidlc package")
            .unwrap();
        assert!(body.contains("${1:name}"));
        assert!(label.starts_with('#'));
    }

    #[test]
    fn placeholder_stripping_leaves_plain_text() {
        assert_eq!(
            strip_snippet_placeholders("#pragma xidlc package ${1:name}"),
            "#pragma xidlc package "
        );
        assert_eq!(strip_snippet_placeholders("#pragma $0"), "#pragma ");
        assert_eq!(strip_snippet_placeholders("plain"), "plain");
    }

    #[test]
    fn custom_types_and_annotations_are_collected() {
        let source = r#"
module M {
    struct S { long x; };
    struct T;
    union U switch (long) { case 1: long a; };
    enum E { A, B };
    bitmask BM { flag0 };
    bitset BS : long { bitfield<1> f; };
    exception Ex { long code; };
    typedef sequence<long> LongSeq;
    interface I : S { void op(in long x); };
    interface F;
    @annotation MyAnno { long v; };
    struct S2 { @MyAnno long x; };
};
@MyAnno
@key
"#;
        let symbols = collect_completion_symbols(source);
        for expected in [
            "S", "T", "U", "E", "BM", "BS", "Ex", "LongSeq", "I", "F", "S2",
        ] {
            assert!(
                symbols.types.iter().any(|name| name == expected),
                "missing custom type {expected}: {:?}",
                symbols.types
            );
        }
        assert_eq!(symbols.annotations, vec!["MyAnno"]);
    }
}
