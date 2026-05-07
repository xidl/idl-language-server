use dashmap::DashMap;
use ropey::Rope;
use tower_lsp::Client;
use tower_lsp::lsp_types::{ConfigurationItem, SemanticToken};

use crate::http_client::PreviewHandle;

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
