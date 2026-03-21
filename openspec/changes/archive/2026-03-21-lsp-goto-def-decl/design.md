## Context

The IDL language server currently lacks go-to navigation for definitions, declarations, and usages. The existing codebase already uses tree-sitter queries for document symbols, diagnostics, and folding. We can extend that approach to capture definition and declaration nodes and map LSP requests accordingly.

## Goals / Non-Goals

**Goals:**
- Add `textDocument/definition`, `textDocument/declaration`, and `textDocument/references`.
- Mark definition nodes as `definition` and type-usage nodes as `declaration` via queries.
- Allow go-to only from declarations to definitions; definitions only support finding usages.

**Non-Goals:**
- Cross-file indexing or workspace-wide symbol resolution.
- Semantic type checking beyond name matching.
- Incremental caches or background indexing.

## Decisions

- Use tree-sitter queries to tag `definition` and `declaration` captures, keeping extraction consistent with other LSP features.
- Resolve navigation by matching symbol names from captures within the same document.
- Restrict go-to: `declaration` → `definition` only; definitions do not go to declarations but can resolve references.

## Risks / Trade-offs

- [Risk] Name-based matching may be ambiguous in the presence of scopes → Mitigation: start with same-document matching; consider adding scope awareness later.
- [Risk] Overlapping captures may misclassify nodes → Mitigation: keep query patterns tight and adjust if necessary.
- [Trade-off] No cross-file navigation → acceptable for initial feature scope.
