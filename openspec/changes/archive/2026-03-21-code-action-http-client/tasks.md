## 1. Detection And Code Action

- [x] 1.1 Add tree-sitter/query helpers to detect HTTP annotations and `pragma xidlc service`
- [x] 1.2 Implement code action on `interface` nodes with `start/stop http client` label based on running state
- [x] 1.3 Add hover action to surface scalar UI URL when preview is running

## 2. Preview Lifecycle

- [x] 2.1 Add state to track running preview per document/workspace
- [x] 2.2 Invoke `xidlc` to generate OpenAPI on start and on document change (debounced)
- [x] 2.3 Stop preview server cleanly when action toggles to stop

## 3. Redoc Server

- [x] 3.1 Vendor scalar assets and add Axum + utoipa-scalar server module (based on client-daemon reference)
- [x] 3.2 Serve generated OpenAPI via the scalar UI endpoint
- [x] 3.3 Wire server startup/shutdown to LSP lifecycle
