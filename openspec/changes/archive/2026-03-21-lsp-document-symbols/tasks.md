## 1. Query And Parsing Setup

- [x] 1.1 Locate or add the tree-sitter-idl query file used for document symbols
- [x] 1.2 Implement a query that captures `struct`, `enum`, `interface`,
      `bitmask`, and `op_dcl` nodes
- [x] 1.3 Add unit coverage or fixtures to validate query captures (if a test
      harness exists)

## 2. Symbol Extraction

- [x] 2.1 Implement a symbol-extraction routine that maps captures to LSP
      `DocumentSymbol`
- [x] 2.2 Build hierarchy by attaching `op_dcl` symbols under their enclosing
      `interface`
- [x] 2.3 Map declaration kinds to appropriate `SymbolKind` values consistently

## 3. LSP Handler Integration

- [x] 3.1 Wire `textDocument/documentSymbol` request to the extraction routine
- [x] 3.2 Ensure the handler uses the current full parse tree (no incremental
      cache)
- [x] 3.3 Add/update tests for the LSP response shape and hierarchy
