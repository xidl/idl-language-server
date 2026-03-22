use tower_lsp::lsp_types::SemanticTokenType;

pub const LANGUAGE_ID: &str = "idl";

pub const TOKEN_TYPE_NAMESPACE: u32 = 0;
pub const TOKEN_TYPE_TYPE: u32 = 1;
#[allow(dead_code)]
pub const TOKEN_TYPE_CLASS: u32 = 2;
#[allow(dead_code)]
pub const TOKEN_TYPE_ENUM: u32 = 3;
#[allow(dead_code)]
pub const TOKEN_TYPE_INTERFACE: u32 = 4;
#[allow(dead_code)]
pub const TOKEN_TYPE_STRUCT: u32 = 5;
#[allow(dead_code)]
pub const TOKEN_TYPE_TYPE_PARAMETER: u32 = 6;
pub const TOKEN_TYPE_PARAMETER: u32 = 7;
pub const TOKEN_TYPE_VARIABLE: u32 = 8;
pub const TOKEN_TYPE_PROPERTY: u32 = 9;
pub const TOKEN_TYPE_ENUM_MEMBER: u32 = 10;
#[allow(dead_code)]
pub const TOKEN_TYPE_EVENT: u32 = 11;
pub const TOKEN_TYPE_FUNCTION: u32 = 12;
pub const TOKEN_TYPE_METHOD: u32 = 13;
pub const TOKEN_TYPE_MACRO: u32 = 14;
pub const TOKEN_TYPE_KEYWORD: u32 = 15;
pub const TOKEN_TYPE_MODIFIER: u32 = 16;
pub const TOKEN_TYPE_COMMENT: u32 = 17;
pub const TOKEN_TYPE_STRING: u32 = 18;
pub const TOKEN_TYPE_NUMBER: u32 = 19;
pub const TOKEN_TYPE_REGEXP: u32 = 20;
pub const TOKEN_TYPE_OPERATOR: u32 = 21;
pub const TOKEN_TYPE_DECORATOR: u32 = 22;

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
    vec![
        SemanticTokenType::new("namespace"),
        SemanticTokenType::new("type"),
        SemanticTokenType::new("class"),
        SemanticTokenType::new("enum"),
        SemanticTokenType::new("interface"),
        SemanticTokenType::new("struct"),
        SemanticTokenType::new("typeParameter"),
        SemanticTokenType::new("parameter"),
        SemanticTokenType::new("variable"),
        SemanticTokenType::new("property"),
        SemanticTokenType::new("enumMember"),
        SemanticTokenType::new("event"),
        SemanticTokenType::new("function"),
        SemanticTokenType::new("method"),
        SemanticTokenType::new("macro"),
        SemanticTokenType::new("keyword"),
        SemanticTokenType::new("modifier"),
        SemanticTokenType::new("comment"),
        SemanticTokenType::new("string"),
        SemanticTokenType::new("number"),
        SemanticTokenType::new("regexp"),
        SemanticTokenType::new("operator"),
        SemanticTokenType::new("decorator"),
    ]
}

pub fn capture_to_token_type(capture: &str) -> Option<u32> {
    match capture {
        "comment.documentation"
        | "comment.error"
        | "comment.warning"
        | "comment.todo"
        | "comment.note"
        | "comment" => Some(TOKEN_TYPE_COMMENT),
        "string.documentation"
        | "string.escape"
        | "string.special"
        | "string.special.symbol"
        | "string.special.path"
        | "string.special.url"
        | "string" => Some(TOKEN_TYPE_STRING),
        "string.regexp" => Some(TOKEN_TYPE_REGEXP),
        "character.special" | "character" => Some(TOKEN_TYPE_STRING),
        "number.float" | "number" | "boolean" => Some(TOKEN_TYPE_NUMBER),
        "keyword.modifier" => Some(TOKEN_TYPE_MODIFIER),
        "keyword.directive" => Some(TOKEN_TYPE_MACRO),
        "keyword.directive.define"
        | "keyword.conditional"
        | "keyword.conditional.ternary"
        | "keyword.exception"
        | "keyword.import"
        | "keyword.operator"
        | "keyword.coroutine"
        | "keyword.function"
        | "keyword.repeat"
        | "keyword.return"
        | "keyword.debug"
        | "keyword.type"
        | "keyword" => Some(TOKEN_TYPE_KEYWORD),
        "type.builtin" | "type.definition" | "type" | "tag" | "tag.builtin" | "tag.attribute"
        | "tag.delimiter" => Some(TOKEN_TYPE_TYPE),
        "function.macro" => Some(TOKEN_TYPE_MACRO),
        "function.method" | "function.method.call" => Some(TOKEN_TYPE_METHOD),
        "function.builtin" | "function.call" | "function" => Some(TOKEN_TYPE_FUNCTION),
        "variable.parameter" | "variable.parameter.builtin" => Some(TOKEN_TYPE_PARAMETER),
        "variable.member" | "property" => Some(TOKEN_TYPE_PROPERTY),
        "variable.builtin" | "variable" | "label" => Some(TOKEN_TYPE_VARIABLE),
        "constant.macro" => Some(TOKEN_TYPE_MACRO),
        "constant.builtin" | "constant" => Some(TOKEN_TYPE_ENUM_MEMBER),
        "operator"
        | "punctuation.delimiter"
        | "punctuation.bracket"
        | "punctuation.special"
        | "punctuation" => Some(TOKEN_TYPE_OPERATOR),
        "module.builtin" | "module" => Some(TOKEN_TYPE_NAMESPACE),
        "attribute.builtin" | "attribute" => Some(TOKEN_TYPE_DECORATOR),
        "markup" | "diff" => None,
        _ => None,
    }
}
