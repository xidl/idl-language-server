use std::collections::HashMap;
use std::sync::OnceLock;

use log::warn;
use rust_embed::RustEmbed;
use serde::Deserialize;

#[derive(RustEmbed)]
#[folder = "snippets"]
struct Snippets;

/// A single VSCode-style snippet loaded from the embedded `snippets/` folder.
#[derive(Debug, Clone)]
pub(crate) struct Snippet {
    pub(crate) prefixes: Vec<String>,
    pub(crate) body: String,
    pub(crate) description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Prefix {
    One(String),
    Many(Vec<String>),
}

impl Prefix {
    fn into_vec(self) -> Vec<String> {
        match self {
            Prefix::One(one) => vec![one],
            Prefix::Many(many) => many,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Body {
    One(String),
    Many(Vec<String>),
}

impl Body {
    fn into_string(self) -> String {
        match self {
            Body::One(one) => one,
            Body::Many(many) => many.join("\n"),
        }
    }
}

#[derive(Debug, Deserialize)]
struct VscodeSnippet {
    prefix: Option<Prefix>,
    body: Option<Body>,
    description: Option<String>,
}

/// All snippets embedded at compile time, parsed once at runtime.
pub(crate) fn all() -> &'static [Snippet] {
    static SNIPPETS: OnceLock<Vec<Snippet>> = OnceLock::new();
    SNIPPETS.get_or_init(load_all)
}

fn load_all() -> Vec<Snippet> {
    let mut snippets = Vec::new();
    for path in Snippets::iter() {
        if !path.ends_with(".json") {
            continue;
        }
        let Some(embedded) = Snippets::get(&path) else {
            continue;
        };
        let text = match String::from_utf8(embedded.data.to_vec()) {
            Ok(text) => text,
            Err(err) => {
                warn!("skipping snippet file {path}: not valid UTF-8: {err}");
                continue;
            }
        };
        snippets.extend(parse_snippet_file(&path, &text));
    }
    snippets.sort_by(|left, right| left.prefixes[0].cmp(&right.prefixes[0]));
    snippets
}

fn parse_snippet_file(path: &str, text: &str) -> Vec<Snippet> {
    let entries: HashMap<String, VscodeSnippet> = match serde_json::from_str(text) {
        Ok(entries) => entries,
        Err(err) => {
            warn!("skipping snippet file {path}: invalid JSON: {err}");
            return Vec::new();
        }
    };

    let mut snippets = Vec::new();
    for (name, entry) in entries {
        let Some(body) = entry.body else {
            warn!("skipping snippet {name} in {path}: missing body");
            continue;
        };
        let body = body.into_string();
        let prefixes = entry
            .prefix
            .map(Prefix::into_vec)
            .filter(|prefixes| !prefixes.is_empty())
            .unwrap_or_else(|| vec![name.clone()]);
        snippets.push(Snippet {
            prefixes,
            body,
            description: entry.description,
        });
    }
    snippets
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_snippets_load_at_runtime() {
        let snippets = all();
        assert!(
            !snippets.is_empty(),
            "snippets/ folder should embed snippets"
        );

        let module = snippets
            .iter()
            .find(|snippet| snippet.prefixes.iter().any(|prefix| prefix == "module"))
            .expect("module snippet should exist");
        assert!(
            module.body.contains("${1:name}"),
            "snippet body should keep VSCode placeholders"
        );

        let get = snippets
            .iter()
            .find(|snippet| snippet.prefixes.iter().any(|prefix| prefix == "@get"))
            .expect("@get snippet should exist");
        assert!(
            get.body.contains("path ="),
            "HTTP verb snippet should include a path"
        );
    }

    #[test]
    fn pragma_builtins_are_embedded() {
        let snippets = all();
        for prefix in ["#pragma package", "#pragma version", "#pragma service"] {
            assert!(
                snippets
                    .iter()
                    .any(|snippet| snippet.prefixes.iter().any(|candidate| candidate == prefix)),
                "missing builtin snippet {prefix}"
            );
        }
    }

    #[test]
    fn parses_vscode_snippet_file() {
        let text = r#"{
          "single": { "prefix": "sing", "body": "sing ${1:x};", "description": "single" },
          "multi": {
            "prefix": ["m1", "m2"],
            "body": ["line1 ${1:a}", "line2 $0"],
            "description": "multi line"
          },
          "noPrefix": { "body": "uses name" }
        }"#;
        let parsed = parse_snippet_file("test.json", text);
        assert_eq!(parsed.len(), 3);

        let single = parsed.iter().find(|s| s.prefixes == ["sing"]).unwrap();
        assert_eq!(single.body, "sing ${1:x};");
        assert_eq!(single.description.as_deref(), Some("single"));

        let multi = parsed.iter().find(|s| s.prefixes == ["m1", "m2"]).unwrap();
        assert_eq!(multi.body, "line1 ${1:a}\nline2 $0");

        let no_prefix = parsed.iter().find(|s| s.prefixes == ["noPrefix"]).unwrap();
        assert_eq!(no_prefix.body, "uses name");
    }

    #[test]
    fn invalid_json_is_skipped() {
        assert!(parse_snippet_file("broken.json", "{ not json").is_empty());
    }

    #[test]
    fn entries_without_body_are_skipped() {
        let text = r#"{
          "ok": { "prefix": "ok", "body": "fine" },
          "bad": { "prefix": "bad" }
        }"#;
        let parsed = parse_snippet_file("test.json", text);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].prefixes, ["ok"]);
    }
}
