## Why

The IDL language server currently lacks Document Symbols, which makes outline navigation and symbol search in editors difficult. Adding this feature now unblocks core LSP UX for IDL users with minimal scope.

## What Changes

- Implement Document Symbols response for IDL files using tree-sitter queries.
- Extract symbols for top-level declarations (struct, enum, interface, bitmask) and interface members (op_dcl) with correct hierarchy.
- Return symbol list without incremental updates.

## Capabilities

### New Capabilities
- `document-symbols`: Provide LSP Document Symbols for IDL files using tree-sitter-idl, including hierarchical interface members.

### Modified Capabilities
- None.

## Impact

- Language server LSP request handling for `textDocument/documentSymbol`.
- tree-sitter-idl query definitions and symbol extraction logic.
- Tests or fixtures for symbol extraction (if present).
