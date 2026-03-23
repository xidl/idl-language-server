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

pub const HIGHLIGHT_NAMES: &[&str] = &[
    "comment.documentation",
    "comment.error",
    "comment.warning",
    "comment.todo",
    "comment.note",
    "string.documentation",
    "string.escape",
    "string.regexp",
    "string.special",
    "string.special.symbol",
    "string.special.path",
    "string.special.url",
    "character.special",
    "number.float",
    "type.builtin",
    "type.definition",
    "variable.builtin",
    "variable.parameter",
    "variable.parameter.builtin",
    "variable.member",
    "constant.builtin",
    "constant.macro",
    "function.builtin",
    "function.call",
    "function.macro",
    "function.method",
    "function.method.call",
    "attribute.builtin",
    "module.builtin",
    "keyword.directive",
    "keyword.directive.define",
    "keyword.conditional",
    "keyword.conditional.ternary",
    "keyword.exception",
    "keyword.import",
    "keyword.operator",
    "keyword.coroutine",
    "keyword.function",
    "keyword.modifier",
    "keyword.repeat",
    "keyword.return",
    "keyword.debug",
    "keyword.type",
    "punctuation.delimiter",
    "punctuation.bracket",
    "punctuation.special",
    "comment",
    "string",
    "character",
    "number",
    "boolean",
    "keyword",
    "type",
    "variable",
    "constant",
    "function",
    "attribute",
    "module",
    "property",
    "operator",
    "punctuation",
    "markup",
    "diff",
    "label",
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

pub fn capture_to_token_type(capture: &str) -> Option<u32> {
    match capture {
        "comment.documentation"
        | "comment.error"
        | "comment.warning"
        | "comment.todo"
        | "comment.note"
        | "comment" => token_type_index("comment"),
        "string.documentation"
        | "string.special"
        | "string.special.symbol"
        | "string.special.path"
        | "string.special.url"
        | "string" => token_type_index("string"),
        "string.escape" => token_type_index("escapeSequence"),
        "string.regexp" => token_type_index("regexp"),
        "character.special" | "character" => token_type_index("character"),
        "number.float" | "number" => token_type_index("number"),
        "boolean" => token_type_index("boolean"),
        "keyword.modifier" => token_type_index("keyword"),
        "keyword.directive" => token_type_index("macro"),
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
        | "keyword.type"
        | "keyword" => token_type_index("keyword"),
        "keyword.operator" => token_type_index("operator"),
        "type.builtin" => token_type_index("builtinType"),
        "type.definition" | "type" | "tag" | "tag.builtin" | "tag.attribute" | "tag.delimiter" => {
            token_type_index("type")
        }
        "function.macro" => token_type_index("macro"),
        "function.method" | "function.method.call" => token_type_index("method"),
        "function.builtin" | "function.call" | "function" => token_type_index("function"),
        "variable.parameter" | "variable.parameter.builtin" => token_type_index("parameter"),
        "variable.member" | "property" => token_type_index("property"),
        "variable.builtin" | "variable" => token_type_index("variable"),
        "label" => token_type_index("label"),
        "constant.macro" => token_type_index("macro"),
        "constant.builtin" | "constant" => token_type_index("const"),
        "operator" => token_type_index("operator"),
        "punctuation.delimiter"
        | "punctuation.bracket"
        | "punctuation.special"
        | "punctuation" => token_type_index("punctuation"),
        "module.builtin" | "module" => token_type_index("namespace"),
        "attribute.builtin" => token_type_index("builtinAttribute"),
        "attribute" => token_type_index("attribute"),
        "markup" | "diff" => None,
        _ => None,
    }
}
