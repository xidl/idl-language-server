use std::collections::HashMap;
use std::sync::OnceLock;
use tower_lsp::lsp_types::{SemanticTokenModifier, SemanticTokenType};

pub const LANGUAGE_ID: &str = "idl";

pub const SEMANTIC_TOKEN_TYPES: &[&str] = &[
    // rust-analyzer semantic token types
    "angle",
    "arithmetic",
    "attribute",
    "attributeBracket",
    "bitwise",
    "boolean",
    "brace",
    "bracket",
    "builtinAttribute",
    "builtinType",
    "character",
    "colon",
    "comma",
    "comparison",
    "constParameter",
    "const",
    "derive",
    "deriveHelper",
    "dot",
    "escapeSequence",
    "formatSpecifier",
    "invalidEscapeSequence",
    "label",
    "lifetime",
    "logical",
    "macroBang",
    "parenthesis",
    "procMacro",
    "punctuation",
    "operator",
    "selfKeyword",
    "selfTypeKeyword",
    "semicolon",
    "static",
    "toolModule",
    "typeAlias",
    "union",
    "unresolvedReference",
    // standard LSP semantic token types
    "namespace",
    "type",
    "class",
    "enum",
    "interface",
    "struct",
    "typeParameter",
    "parameter",
    "variable",
    "property",
    "enumMember",
    "event",
    "function",
    "method",
    "macro",
    "keyword",
    "comment",
    "string",
    "number",
    "regexp",
    "decorator",
];

pub const SEMANTIC_TOKEN_MODIFIERS: &[&str] = &[
    "async",
    "attribute",
    "callable",
    "constant",
    "consuming",
    "controlFlow",
    "crateRoot",
    "defaultLibrary",
    "injected",
    "intraDocLink",
    "library",
    "macro",
    "mutable",
    "procMacro",
    "public",
    "reference",
    "trait",
    "unsafe",
];

// https://neovim.io/doc/user/treesitter.html#treesitter-highlight-groups
pub const HIGHLIGHT_NAMES: &[&str] = &[
    "variable",
    "variable.builtin",
    "variable.parameter",
    "variable.parameter.builtin",
    "variable.member",
    "constant",
    "constant.builtin",
    "constant.macro",
    "module",
    "module.builtin",
    "label",
    "string",
    "string.documentation",
    "string.regexp",
    "string.escape",
    "string.special",
    "string.special.symbol",
    "string.special.path",
    "string.special.url",
    "character",
    "character.special",
    "boolean",
    "number",
    "number.float",
    "type",
    "type.builtin",
    "type.definition",
    "attribute",
    "attribute.builtin",
    "property",
    "function",
    "function.builtin",
    "function.call",
    "function.macro",
    "function.method",
    "function.method.call",
    "constructor",
    "operator",
    "keyword",
    "keyword.coroutine",
    "keyword.function",
    "keyword.operator",
    "keyword.import",
    "keyword.type",
    "keyword.modifier",
    "keyword.repeat",
    "keyword.return",
    "keyword.debug",
    "keyword.exception",
    "keyword.conditional",
    "keyword.conditional.ternary",
    "keyword.directive",
    "keyword.directive.define",
    "punctuation.delimiter",
    "punctuation.bracket",
    "punctuation.special",
    "comment",
    "comment.documentation",
    "comment.error",
    "comment.warning",
    "comment.todo",
    "comment.note",
    "markup.strong",
    "markup.italic",
    "markup.strikethrough",
    "markup.underline",
    "markup.heading",
    "markup.heading.1",
    "markup.heading.2",
    "markup.heading.3",
    "markup.heading.4",
    "markup.heading.5",
    "markup.heading.6",
    "markup.quote",
    "markup.math",
    "markup.link",
    "markup.link.label",
    "markup.link.url",
    "markup.raw",
    "markup.raw.block",
    "markup.list",
    "markup.list.checked",
    "markup.list.unchecked",
    "diff.plus",
    "diff.minus",
    "diff.delta",
    "tag",
    "tag.builtin",
    "tag.attribute",
    "tag.delimiter",
];

pub const DOCUMENT_SYMBOL_QUERY: &str = include_str!("../queries/document_symbols.scm");
pub const DIAGNOSTICS_QUERY: &str = include_str!("../queries/diagnostics.scm");
pub const FOLDING_QUERY: &str = include_str!("../queries/folding_ranges.scm");
pub const GOTO_QUERY: &str = include_str!("../queries/goto_symbols.scm");

pub const DIAGNOSTIC_SOURCE: &str = "idl-language-server";

pub const TITLE_START_HTTP_PREVIEW: &str = "$(play) Start HTTP API Preview";
pub const TITLE_STOP_HTTP_PREVIEW: &str = "$(debug-stop) Stop HTTP API Preview";
pub const TITLE_OPEN_HTTP_PREVIEW: &str = "$(link-external) Open HTTP API Preview";
pub const TITLE_STOP_HTTP_CLIENT: &str = "$(debug-stop) Stop HTTP Preview";

pub const MSG_MISSING_DOCUMENT_URI: &str = "Missing document URI";
pub const MSG_DOCUMENT_NOT_AVAILABLE: &str = "Document not available";

pub const COMMAND_VSCODE_OPEN: &str = "vscode.open";
pub const COMMAND_INSPECT_HIR: &str = "idl-language-server.inspect-hir";
pub const COMMAND_INSPECT_TYPEDAST: &str = "idl-language-server.inspect-typedast";

pub fn semantic_token_types() -> Vec<SemanticTokenType> {
    SEMANTIC_TOKEN_TYPES
        .iter()
        .map(|token_type| SemanticTokenType::new(*token_type))
        .collect()
}

pub fn semantic_token_modifiers() -> Vec<SemanticTokenModifier> {
    SEMANTIC_TOKEN_MODIFIERS
        .iter()
        .map(|modifier| SemanticTokenModifier::new(*modifier))
        .collect()
}

fn token_type_index(token_type: &str) -> Option<u32> {
    static TOKEN_TYPE_MAP: OnceLock<HashMap<&'static str, u32>> = OnceLock::new();
    let map = TOKEN_TYPE_MAP.get_or_init(|| {
        SEMANTIC_TOKEN_TYPES
            .iter()
            .enumerate()
            .map(|(index, &name)| (name, index as u32))
            .collect()
    });
    map.get(token_type).copied()
}

fn token_modifier_index(modifier: &str) -> Option<u32> {
    static TOKEN_MODIFIER_MAP: OnceLock<HashMap<&'static str, u32>> = OnceLock::new();
    let map = TOKEN_MODIFIER_MAP.get_or_init(|| {
        SEMANTIC_TOKEN_MODIFIERS
            .iter()
            .enumerate()
            .map(|(index, &name)| (name, index as u32))
            .collect()
    });
    map.get(modifier).copied()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SemanticTokenInfo {
    pub token_type: u32,
    pub token_modifiers_bitset: u32,
}

fn token(token_type: &str, modifiers: &[&str]) -> Option<SemanticTokenInfo> {
    let token_type = token_type_index(token_type)?;
    let mut token_modifiers_bitset = 0u32;

    for modifier in modifiers {
        let index = token_modifier_index(modifier)?;
        token_modifiers_bitset |= 1 << index;
    }

    Some(SemanticTokenInfo {
        token_type,
        token_modifiers_bitset,
    })
}

pub fn capture_to_semantic_token(capture: &str) -> Option<SemanticTokenInfo> {
    match capture {
        "comment.documentation"
        | "comment.block"
        | "comment.block.documentation"
        | "comment.line"
        | "comment.line.documentation"
        | "comment.error"
        | "comment.warning"
        | "comment.todo"
        | "comment.note"
        | "comment" => token("comment", &[]),
        "string.documentation"
        | "string.special"
        | "string.special.symbol"
        | "string.special.path"
        | "string.special.url"
        | "string" => token("string", &[]),
        "string.escape" => token("escapeSequence", &[]),
        "string.regexp" => token("regexp", &[]),
        "character.special" | "character" => token("character", &[]),
        "number.float" | "number" => token("number", &[]),
        "boolean" => token("boolean", &[]),
        "keyword.modifier" => token("keyword", &[]),
        "keyword.directive" => token("macro", &[]),
        "keyword.directive.define"
        | "keyword.conditional"
        | "keyword.conditional.ternary"
        | "keyword.exception"
        | "keyword.import"
        | "keyword.coroutine"
        | "keyword.function"
        | "keyword.repeat"
        | "keyword.return"
        | "keyword.debug"
        | "keyword" => token("keyword", &[]),
        "keyword.type" => token("keyword", &[]),
        "keyword.operator" => token("operator", &[]),
        "type.builtin" => token("type", &["defaultLibrary"]),
        "type.definition" | "type" | "tag" | "tag.builtin" | "tag.attribute" | "tag.delimiter" => {
            token("type", &[])
        }
        "function.macro" => token("macro", &[]),
        "function.method" | "function.method.call" => token("method", &[]),
        "function.builtin" => token("function", &["defaultLibrary"]),
        "function.call" | "function" => token("function", &[]),
        "variable.parameter" => token("parameter", &[]),
        "variable.parameter.builtin" => token("parameter", &["defaultLibrary"]),
        "variable.member" | "property" => token("property", &[]),
        "variable.builtin" => token("variable", &["defaultLibrary"]),
        "variable" => token("variable", &[]),
        "label" => token("label", &[]),
        "constant.macro" => token("macro", &[]),
        "constant.builtin" => token("const", &["defaultLibrary"]),
        "constant" => token("const", &[]),
        "operator" => token("operator", &[]),
        "punctuation.delimiter" | "punctuation.bracket" | "punctuation.special" | "punctuation" => {
            token("punctuation", &[])
        }
        "module.builtin" => token("namespace", &["defaultLibrary"]),
        "module" => token("namespace", &[]),
        "attribute.builtin" => token("property", &["defaultLibrary"]),
        "attribute" => token("property", &[]),
        "markup" | "diff" => None,
        _ => None,
    }
}
