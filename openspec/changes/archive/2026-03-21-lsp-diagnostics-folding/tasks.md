## 1. Diagnostics Query And Publication

- [x] 1.1 Add a tree-sitter query to capture `ERROR` nodes for diagnostics
- [x] 1.2 Implement diagnostic extraction from `ERROR` captures and map to LSP
      diagnostics
- [x] 1.3 Publish diagnostics on document open/change using the latest full
      parse tree

## 2. Folding Ranges

- [x] 2.1 Add a tree-sitter query to capture foldable `interface` and `module`
      nodes
- [x] 2.2 Implement folding range extraction and map to LSP `FoldingRange`
- [x] 2.3 Wire the `textDocument/foldingRange` handler and register capability

## 3. Tests

- [x] 3.1 Add/update tests or fixtures for diagnostics from `ERROR` nodes (if
      test harness exists)
- [x] 3.2 Add/update tests for folding ranges over `interface` and `module`
      blocks
