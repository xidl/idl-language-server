use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use axum::{
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
use tokio::time::Instant;
use tower_lsp::lsp_types::Position;
use tree_sitter::{Parser, Query, QueryCursor, StreamingIterator};
use utoipa::openapi::OpenApi;
use utoipa_scalar::{Scalar, Servable};

use crate::doc::http as http_annotations;
use crate::{node_range, position_in_range};
use ropey::Rope;

const HTTP_ANNOTATION_QUERY: &str = "(annotation) @annotation";
const INTERFACE_QUERY: &str = "(interface_def (interface_header (identifier) @interface.name)) @interface";
const INTERFACE_NAME_QUERY: &str =
    "(interface_def (interface_header (identifier) @interface.name)) @interface";
const SCALAR_HTML: &str = include_str!(concat!(env!("OUT_DIR"), "/scalar.standalone.html"));

pub const CMD_START_HTTP_CLIENT: &str = "idl.httpClient.start";
pub const CMD_STOP_HTTP_CLIENT: &str = "idl.httpClient.stop";

#[derive(Debug)]
pub struct PreviewHandle {
    pub scalar_url: String,
    regen_tx: mpsc::Sender<String>,
    shutdown_tx: oneshot::Sender<()>,
    _server_task: JoinHandle<()>,
    _regen_task: JoinHandle<()>,
    _working_dir: PathBuf,
}

impl PreviewHandle {
    pub fn request_regen(&self, text: &str) {
        let _ = self.regen_tx.try_send(text.to_string());
    }

    pub fn stop(self) {
        let _ = self.shutdown_tx.send(());
    }
}

#[derive(Clone, Debug)]
struct PreviewState {
    openapi_path: PathBuf,
}

pub fn document_is_http_relevant(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    contains_http_annotations(text) || lower.contains("pragma xidlc service")
}

pub fn interface_at_position(text: &str, rope: &Rope, position: Position) -> bool {
    let mut parser = Parser::new();
    if parser.set_language(&tree_sitter_idl::language()).is_err() {
        return false;
    }
    let tree = match parser.parse(text, None) {
        Some(tree) => tree,
        None => return false,
    };
    let query = match Query::new(&tree_sitter_idl::language(), INTERFACE_QUERY) {
        Ok(query) => query,
        Err(_) => return false,
    };

    let capture_names = query.capture_names();
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), text.as_bytes());

    while let Some(m) = matches.next() {
        for capture in m.captures {
            let capture_name = match capture_names.get(capture.index as usize) {
                Some(name) => *name,
                None => continue,
            };
            if capture_name != "interface" {
                continue;
            }
            let range = node_range(capture.node, rope);
            if position_in_range(position, range) {
                return true;
            }
        }
    }
    let line_idx = position.line as usize;
    if line_idx < rope.len_lines() {
        if let Some(line) = rope.get_line(line_idx) {
            let line_text = line.to_string();
            if line_text.contains("interface") {
                return true;
            }
        }
    }
    false
}

pub fn interface_name_positions(text: &str, rope: &Rope) -> Vec<Position> {
    let mut parser = Parser::new();
    if parser.set_language(&tree_sitter_idl::language()).is_err() {
        return Vec::new();
    }
    let tree = match parser.parse(text, None) {
        Some(tree) => tree,
        None => return Vec::new(),
    };
    let query = match Query::new(&tree_sitter_idl::language(), INTERFACE_NAME_QUERY) {
        Ok(query) => query,
        Err(_) => return Vec::new(),
    };

    let capture_names = query.capture_names();
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), text.as_bytes());
    let mut positions = Vec::new();

    while let Some(m) = matches.next() {
        for capture in m.captures {
            let capture_name = match capture_names.get(capture.index as usize) {
                Some(name) => *name,
                None => continue,
            };
            if capture_name != "interface.name" {
                continue;
            }
            let range = node_range(capture.node, rope);
            positions.push(range.end);
        }
    }

    positions
}

pub fn document_has_interface(text: &str) -> bool {
    let mut parser = Parser::new();
    if parser.set_language(&tree_sitter_idl::language()).is_err() {
        return false;
    }
    let tree = match parser.parse(text, None) {
        Some(tree) => tree,
        None => return false,
    };
    let query = match Query::new(&tree_sitter_idl::language(), INTERFACE_QUERY) {
        Ok(query) => query,
        Err(_) => return false,
    };
    let capture_names = query.capture_names();
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), text.as_bytes());
    while let Some(m) = matches.next() {
        for capture in m.captures {
            let capture_name = match capture_names.get(capture.index as usize) {
                Some(name) => *name,
                None => continue,
            };
            if capture_name == "interface" {
                return true;
            }
        }
    }
    false
}

pub async fn start_preview(text: &str) -> anyhow::Result<PreviewHandle> {
    let working_dir = create_working_dir()?;
    let source_path = working_dir.join("source.idl");
    let out_dir = working_dir.join("out");
    tokio::fs::create_dir_all(&out_dir).await?;

    regenerate_openapi(text, &source_path, &out_dir).await?;
    let openapi_path = out_dir.join("openapi.json");
    let openapi = load_openapi(&openapi_path).await?;

    let state = Arc::new(PreviewState { openapi_path });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let scalar_url = format!("http://{}/scalar", addr);

    let app = build_router(state.clone(), openapi);

    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let server_task = tokio::spawn(async move {
        let _ = axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                let _ = shutdown_rx.await;
            })
            .await;
    });

    let (regen_tx, regen_rx) = mpsc::channel(8);
    let regen_task = tokio::spawn(run_regen_loop(regen_rx, source_path, out_dir));

    Ok(PreviewHandle {
        scalar_url,
        regen_tx,
        shutdown_tx,
        _server_task: server_task,
        _regen_task: regen_task,
        _working_dir: working_dir,
    })
}

async fn run_regen_loop(
    mut regen_rx: mpsc::Receiver<String>,
    source_path: PathBuf,
    out_dir: PathBuf,
) {
    while let Some(mut text) = regen_rx.recv().await {
        let delay = tokio::time::sleep(Duration::from_millis(300));
        tokio::pin!(delay);
        loop {
            tokio::select! {
                _ = &mut delay => break,
                next = regen_rx.recv() => {
                    match next {
                        Some(new_text) => {
                            text = new_text;
                            delay.as_mut().reset(Instant::now() + Duration::from_millis(300));
                        }
                        None => break,
                    }
                }
            }
        }

        if let Err(err) = regenerate_openapi(&text, &source_path, &out_dir).await {
            log::warn!("failed to regenerate openapi: {err}");
        }
    }
}

fn build_router(state: Arc<PreviewState>, openapi: OpenApi) -> Router {
    let openapi_state = state.clone();
    let app = Router::new().route(
        "/openapi.json",
        get(move || openapi_json_handler(openapi_state.clone())),
    );

    let scalar = Scalar::with_url("/scalar", openapi).custom_html(SCALAR_HTML);

    app.merge(scalar).with_state(state)
}

async fn openapi_json_handler(state: Arc<PreviewState>) -> Response {
    match tokio::fs::read_to_string(&state.openapi_path).await {
        Ok(body) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/json")],
            body,
        )
            .into_response(),
        Err(err) => {
            log::warn!("failed to read openapi.json: {err}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn regenerate_openapi(
    text: &str,
    source_path: &Path,
    out_dir: &Path,
) -> anyhow::Result<()> {
    tokio::fs::write(source_path, text).await?;
    let status = tokio::process::Command::new("xidlc")
        .arg("--lang")
        .arg("openapi")
        .arg("--out-dir")
        .arg(out_dir)
        .arg(source_path)
        .status()
        .await?;
    if !status.success() {
        anyhow::bail!("xidlc openapi generation failed");
    }
    Ok(())
}

async fn load_openapi(path: &Path) -> anyhow::Result<OpenApi> {
    let content = tokio::fs::read_to_string(path).await?;
    let openapi: OpenApi = serde_json::from_str(&content)?;
    Ok(openapi)
}

fn contains_http_annotations(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    for name in http_annotations::HTTP_ANNOTATIONS {
        let needle = format!("@{name}");
        if lower.contains(&needle) {
            return true;
        }
    }

    let mut parser = Parser::new();
    if parser.set_language(&tree_sitter_idl::language()).is_err() {
        return false;
    }
    let tree = match parser.parse(text, None) {
        Some(tree) => tree,
        None => return false,
    };
    let query = match Query::new(&tree_sitter_idl::language(), HTTP_ANNOTATION_QUERY) {
        Ok(query) => query,
        Err(_) => return false,
    };

    let capture_names = query.capture_names();
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), text.as_bytes());

    while let Some(m) = matches.next() {
        for capture in m.captures {
            let capture_name = match capture_names.get(capture.index as usize) {
                Some(name) => *name,
                None => continue,
            };
            if capture_name != "annotation" {
                continue;
            }
            let raw = match capture.node.utf8_text(text.as_bytes()) {
                Ok(text) => text.trim(),
                Err(_) => continue,
            };
            if !raw.starts_with('@') {
                continue;
            }
            let name = raw
                .trim_start_matches('@')
                .split(|c: char| c == '(' || c.is_whitespace())
                .next()
                .unwrap_or("")
                .to_ascii_lowercase();
            if http_annotations::is_http_annotation(&name) {
                return true;
            }
        }
    }

    false
}

fn create_working_dir() -> anyhow::Result<PathBuf> {
    let base = std::env::temp_dir();
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let dir = base.join(format!("idl-http-preview-{ts}"));
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}
