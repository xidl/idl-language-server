use dashmap::DashMap;
use ropey::Rope;
use tower_lsp::lsp_types::SemanticToken;
use tower_lsp::Client;

use crate::http_client::PreviewHandle;

#[derive(Debug)]
pub struct AppContext {
    pub(crate) client: Client,
    pub(crate) document_map: DashMap<String, Rope>,
    pub(crate) semantic_tokens_map: DashMap<String, Vec<SemanticToken>>,
    pub(crate) preview_map: DashMap<String, PreviewHandle>,
}

impl AppContext {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            document_map: DashMap::new(),
            semantic_tokens_map: DashMap::new(),
            preview_map: DashMap::new(),
        }
    }
}
