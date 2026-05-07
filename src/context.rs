use dashmap::DashMap;
use ropey::Rope;
use serde::{Deserialize, Serialize};
use tower_lsp::Client;
use tower_lsp::lsp_types::{ConfigurationItem, SemanticToken};

use crate::http_client::PreviewHandle;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Plugin {
    #[serde(alias = "hir")]
    Hir,
    #[serde(alias = "rest_hir", alias = "rest-hir")]
    RestHir,
    #[serde(alias = "typed_ast", alias = "typed-ast")]
    TypedAst,
    #[serde(alias = "rs", alias = "rust")]
    Rust,
    #[serde(
        alias = "rust_jsonrpc",
        alias = "rust-jsonrpc",
        alias = "rs_jsonrpc",
        alias = "rs-jsonrpc"
    )]
    RustJsonRpc,
    #[serde(
        alias = "axum",
        alias = "rust_axum",
        alias = "rust-axum",
        alias = "rs_axum",
        alias = "rs-axum"
    )]
    Axum,
    #[serde(alias = "ts", alias = "typescript")]
    Typescript,
    #[serde(
        alias = "ts_rest",
        alias = "ts-rest",
        alias = "typescript_rest",
        alias = "typescript-rest"
    )]
    TypescriptRest,
    #[serde(alias = "go", alias = "golang")]
    Go,
    #[serde(alias = "go_rest", alias = "go-rest")]
    GoRest,
    #[serde(alias = "py", alias = "python")]
    Python,
    #[serde(
        alias = "py_rest",
        alias = "py-rest",
        alias = "python_rest",
        alias = "python-rest"
    )]
    PythonRest,
    #[serde(alias = "openapi")]
    Openapi,
    #[serde(alias = "openrpc", alias = "open-rpc")]
    Openrpc,
}

impl Plugin {
    pub fn to_xidlc_cmd(self) -> &'static str {
        match self {
            Plugin::Hir => "hir",
            Plugin::RestHir => "rest-hir",
            Plugin::TypedAst => "typed-ast",
            Plugin::Rust => "rust",
            Plugin::RustJsonRpc => "rust-jsonrpc",
            Plugin::Axum => "rust-axum",
            Plugin::Typescript => "typescript",
            Plugin::TypescriptRest => "typescript-rest",
            Plugin::Go => "go",
            Plugin::GoRest => "go-rest",
            Plugin::Python => "python",
            Plugin::PythonRest => "python-rest",
            Plugin::Openapi => "openapi",
            Plugin::Openrpc => "openrpc",
        }
    }

    pub fn to_language_id(self) -> &'static str {
        match self {
            Plugin::Hir
            | Plugin::RestHir
            | Plugin::TypedAst
            | Plugin::Openapi
            | Plugin::Openrpc => "json",
            Plugin::Rust | Plugin::RustJsonRpc | Plugin::Axum => "rust",
            Plugin::Typescript | Plugin::TypescriptRest => "typescript",
            Plugin::Go | Plugin::GoRest => "go",
            Plugin::Python | Plugin::PythonRest => "python",
        }
    }
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct Settings {
    #[serde(rename = "xidlcPath")]
    pub xidlc_path: String,
    #[serde(rename = "httpClient.regenerateCommand")]
    pub regenerate_command: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            xidlc_path: "xidlc".to_string(),
            regenerate_command: "{xidlc_path} gen --out-dir {out_dir} openapi {source_path}"
                .to_string(),
        }
    }
}

#[derive(Debug)]
pub struct AppContext {
    pub(crate) client: Client,
    pub(crate) document_map: DashMap<String, Rope>,
    pub(crate) semantic_tokens_map: DashMap<String, Vec<SemanticToken>>,
    pub(crate) preview_map: DashMap<String, PreviewHandle>,
    pub(crate) settings: tokio::sync::RwLock<Settings>,
}

impl AppContext {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            document_map: DashMap::new(),
            semantic_tokens_map: DashMap::new(),
            preview_map: DashMap::new(),
            settings: tokio::sync::RwLock::new(Settings::default()),
        }
    }

    pub async fn fetch_settings(&self) {
        let items = vec![ConfigurationItem {
            scope_uri: None,
            section: Some("idl-language-server".to_string()),
        }];

        if let Ok(configs) = self.client.configuration(items).await {
            if let Some(config) = configs.first() {
                if let Ok(settings) = serde_json::from_value::<Settings>(config.clone()) {
                    let mut w = self.settings.write().await;
                    *w = settings;
                }
            }
        }
    }
}
