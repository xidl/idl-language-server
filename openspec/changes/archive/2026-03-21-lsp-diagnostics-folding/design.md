## Context

The IDL language server already parses documents with tree-sitter but does not
expose parse errors as diagnostics and does not provide folding ranges. The
earlier document-symbols work introduced a query-driven approach for extracting
syntax nodes. We can reuse the same tree-sitter parsing strategy and add two new
LSP endpoints: diagnostics publication and folding ranges.

## Goals / Non-Goals

**Goals:**

- Emit diagnostics for every tree-sitter `ERROR` node in the current parse tree.
- Provide folding ranges for `interface` and `module` declarations using a
  query.
- Keep the implementation non-incremental and query-driven for simplicity.

**Non-Goals:**

- Full semantic validation beyond parse errors.
- Incremental diagnostics updates or caching.
- Folding for every possible construct (only `interface` and `module`).

## Decisions

- Use a tree-sitter query to capture `ERROR` nodes for diagnostics instead of
  manual tree traversal, keeping extraction consistent with other query-based
  features.
- Use a separate query to capture foldable `interface` and `module` nodes and
  map them to LSP `FoldingRange`.
- Publish diagnostics on document open/change using the existing full-text parse
  (no incremental state).

## Risks / Trade-offs

- [Risk] Tree-sitter `ERROR` nodes may be noisy or overlap; diagnostics could be
  verbose → Mitigation: keep severity as `Error` and rely on client filtering;
  consider future deduping if needed.
- [Risk] Folding ranges may be slightly misaligned with user expectations →
  Mitigation: use node byte ranges and map to line-based ranges; adjust query if
  needed.
- [Trade-off] Non-incremental parsing may be slower on very large files →
  acceptable for current scope.
