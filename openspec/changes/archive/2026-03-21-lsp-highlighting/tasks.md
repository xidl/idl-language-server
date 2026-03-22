## 1. Dependencies and Wiring

- [x] 1.1 Add `tree-sitter-idl` and `tree-sitter-highlight` dependencies (and
      any required features) to the LSP crate
- [x] 1.2 Locate existing LSP semantic tokens pipeline (or create module) for
      full-token responses
- [x] 1.3 Add configuration/constants for token types/modifiers aligned with
      `xidlc` highlight categories

## 2. Highlight Engine

- [x] 2.1 Mirror logic from `xidlc/src/diagnostic/highlight.rs` to define
      capture-to-category mapping
- [x] 2.2 Implement tree-sitter parsing and highlight capture extraction for
      full documents
- [x] 2.3 Convert highlight ranges into LSP semantic tokens (full result only,
      no delta)
- [x] 2.4 Handle parsing/highlight errors deterministically (empty token list or
      graceful fallback)

## 3. LSP Integration

- [x] 3.1 Advertise semantic tokens capability (full only) in server
      initialization
- [x] 3.2 On `didOpen` and `didChange`, recompute and publish/serve full
      semantic tokens
- [x] 3.3 Ensure token ordering and range encoding match LSP requirements

## 4. Validation and Quality

- [x] 4.1 Build the workspace and ensure compilation has no warnings
- [x] 4.2 Start the LSP and request semantic tokens for
      `/Users/loongtao/xidl/xidlc-examples/api/http/http_server.idl`
- [x] 4.3 Verify highlight correctness by inspecting returned tokens (via curl)
      against expected syntax
