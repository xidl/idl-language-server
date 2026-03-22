## 1. Plan And Baseline

- [x] 1.1 Inventory responsibilities in `src/main.rs` and map them to target
      modules
- [x] 1.2 Identify shared state and define the context/OO ownership model

## 2. Module Extraction

- [x] 2.1 Create module structure and move constants into a dedicated module
- [x] 2.2 Extract document state and diagnostics logic into a `documents` module
- [x] 2.3 Extract HTTP client/preview handlers into a `http_client` module
      (public API unchanged)
- [x] 2.4 Extract code lens/action/hover handlers into a `lsp_handlers` module
- [x] 2.5 Extract semantic tokens and symbol/folding helpers into a `analysis`
      module

## 3. Context And Wiring

- [x] 3.1 Introduce a context struct that owns shared maps and LSP client
- [x] 3.2 Update handlers to use context accessors instead of passing state
      around
- [x] 3.3 Simplify `main`/server wiring to construct and inject context

## 4. Cleanup And Clippy

- [x] 4.1 Remove hard-coded strings/magic values and replace with constants
- [x] 4.2 Run `cargo clippy` and fix all warnings introduced or existing

## 5. Verification

- [x] 5.1 Sanity-check behavior parity for commands, code lens/action, hover,
      and diagnostics
