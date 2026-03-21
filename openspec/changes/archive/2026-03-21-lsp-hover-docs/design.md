## Context

The IDL language server currently does not provide hover documentation. The project already uses tree-sitter queries for symbol extraction and LSP features, and recent work added go-to definition/references and rename. We can build hover on the same query-based extraction, but render documentation via embedded Jinja templates with runtime context from xidl HIR and symbol lookup utilities.

## Goals / Non-Goals

**Goals:**
- Implement `textDocument/hover` to render documentation based on query-marked nodes (e.g., `@http`).
- Use `rust-embed` to bundle documentation templates and `minijinja` to render them.
- Provide a rendering context that includes doc name/path, symbol HIR, and symbol lookup/reference helpers.
- Allow templates to surface curated external references (e.g., MDN) as links or short excerpts.

**Non-Goals:**
- Full documentation generation or site build.
- Cross-repository doc loading (only embedded assets).
- Advanced semantic analysis beyond existing HIR and reference APIs.

## Decisions

- Use a dedicated tree-sitter query to capture hover-relevant nodes and map them to template names (e.g., `@http` -> `http.md`).
- Embed templates with `rust-embed` and look them up dynamically by name/path at hover time.
- Use `minijinja` as the templating engine and expose Rust-provided context + functions for symbol and reference lookup.
- Keep external references embedded or linked in templates; avoid live fetch in the LSP path.

## Risks / Trade-offs

- [Risk] Query coverage might miss some annotations or aliases → Mitigation: start with known annotations and iterate.
- [Risk] Template rendering failures could degrade hover → Mitigation: return a safe fallback message when template or render fails.
- [Trade-off] Rendering on demand may be slower for large HIR contexts → acceptable for initial scope; optimize later if needed.
