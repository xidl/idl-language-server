pub(super) const HTTP_ANNOTATIONS: &[&str] = &["http", "get", "query"];

pub(super) fn is_http_annotation(name: &str) -> bool {
    HTTP_ANNOTATIONS.contains(&name)
}
