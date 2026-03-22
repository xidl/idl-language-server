## Why

The LSP server implementation is currently concentrated in a single large module, which makes it hard to reason about responsibilities, state flow, and clippy issues. Refactoring into focused modules will reduce coupling, improve maintainability, and make state ownership explicit.

## What Changes

- Split `src/main.rs` into functional modules (e.g., server wiring, request handlers, document state, HTTP preview, diagnostics, code lens/action, semantic tokens).
- Introduce clear context/OO structures to own shared state and avoid ad-hoc state passing.
- Consolidate hard-coded strings and magic values into centralized constants.
- Address all existing `cargo clippy` warnings after refactor.

## Capabilities

### New Capabilities
- `behavior-parity`: ensure the refactor preserves existing LSP external behavior while internal structure changes.

### Modified Capabilities
- None.

## Impact

- `src/main.rs` will be significantly reduced; new modules will be created under `src/`.
- Public LSP behavior should remain unchanged; internal structure and ownership will change.
- Clippy clean-up may touch multiple modules.
