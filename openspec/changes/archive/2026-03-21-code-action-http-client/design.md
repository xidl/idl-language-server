## Context

The IDL language server currently does not expose any code actions to start an
HTTP client preview workflow. Developers must run `xidlc` and start a viewer
manually. The change adds a code action on `interface` declarations when
HTTP-related content is present (annotations from `docs/rfc/http.md` or
`pragma xidlc service`), and manages a background preview server that
regenerates OpenAPI on document changes.

## Goals / Non-Goals

**Goals:**

- Provide a code action on interface nodes to start/stop an HTTP client preview.
- Detect HTTP relevance via annotations tied to `docs/rfc/http.md` and
  `pragma xidlc service`.
- Run `xidlc` to produce OpenAPI and serve it via Axum + utoipa-scalar with
  vendored assets.
- Regenerate OpenAPI when the source document changes while the preview is
  running.
- Provide a hover action to open the scalar UI URL when the preview is running.

**Non-Goals:**

- Full multi-project preview management or persistent daemon process.
- Automatic discovery across workspace roots beyond the current document.
- Dynamic fetching of external assets (all assets are vendored).

## Decisions

- **Code action location**: Use tree-sitter to locate `interface` nodes and
  attach a single action when the current document is HTTP-relevant. This keeps
  the UX consistent and avoids action noise on non-HTTP files.
- **HTTP relevance detection**: Reuse query-based detection for HTTP annotations
  and parse for `pragma xidlc service`. This aligns with existing query-based
  LSP features and is easy to extend.
- **Preview lifecycle**: Maintain a per-document (or per-workspace) in-memory
  state for whether the preview server is running. The action toggles start/stop
  based on this state.
- **OpenAPI regeneration**: On document change events, if a preview is running
  for the document, re-run `xidlc` to regenerate the OpenAPI output and refresh
  the served spec.
- **Server implementation**: Use Axum with utoipa-scalar and vendor the scalar
  assets (no runtime network dependency). Follow the reference implementation in
  `/Users/loongtao/cerberus/crates/client-daemon` for structure.
- **Hover action URL**: When the preview is running, the hover action returns a
  URL pointing to the scalar UI (e.g., `http://127.0.0.1:<port>/scalar`) derived
  from the running server state.

## Risks / Trade-offs

- [Risk] Regenerating on every change could be expensive → Mitigation: debounce
  document changes before invoking `xidlc`.
- [Risk] Running a server inside the LSP may affect responsiveness → Mitigation:
  spawn background tasks and avoid blocking LSP request handling.
- [Risk] Asset vendoring increases repo size → Mitigation: vendor only the
  minimal scalar distribution used by the server.
