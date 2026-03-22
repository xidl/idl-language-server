## 1. Hover Query And Templates

- [x] 1.1 Add tree-sitter hover query to capture documentation markers (e.g.,
      `@http`)
- [x] 1.2 Create embedded hover templates (starting with `http.md`)

## 2. Rendering Pipeline

- [x] 2.1 Add `rust-embed` + `minijinja` dependencies and embed docs directory
- [x] 2.2 Build rendering context (doc name/path, HIR, symbol lookup and
      reference helpers)
- [x] 2.3 Implement template rendering with safe fallback on missing/failed
      templates

## 3. LSP Integration

- [x] 3.1 Wire `textDocument/hover` handler and register capability
- [x] 3.2 Add/update tests or fixtures for hover rendering (if test harness
      exists)
