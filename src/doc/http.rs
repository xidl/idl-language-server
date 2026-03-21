pub(crate) const HTTP_ANNOTATIONS: &[&str] = &[
    "http",
    "get",
    "post",
    "put",
    "patch",
    "delete",
    "head",
    "options",
    "path",
    "consumes",
    "produces",
    "deprecated",
    "query",
    "header",
    "cookie",
    "optional",
    "http_basic",
    "http_bearer",
];

pub(crate) fn is_http_annotation(name: &str) -> bool {
    HTTP_ANNOTATIONS.contains(&name)
}
