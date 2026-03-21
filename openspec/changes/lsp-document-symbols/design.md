## Context

The IDL language server currently provides diagnostics and other basic features but does not implement `textDocument/documentSymbol`. The parser stack already uses tree-sitter and the project includes `tree-sitter-idl`, which exposes query-based captures for syntax nodes. The goal is to add document symbols without incremental updates or workspace-wide indexing.

## Goals / Non-Goals

**Goals:**
- Provide `textDocument/documentSymbol` for IDL files.
- Use a tree-sitter query to capture symbols for `struct`, `enum`, `interface`, `bitmask`, and `op_dcl`.
- Return hierarchical symbols where `op_dcl` are children of their enclosing `interface`.
- Keep the implementation non-incremental and straightforward.

**Non-Goals:**
- Workspace symbols or cross-file indexing.
- Incremental symbol updates.
- Exhaustive symbol coverage beyond the requested node types.

## Decisions

- Use a dedicated tree-sitter query in `tree-sitter-idl` (or query file in the repo) to capture the required nodes, rather than ad-hoc tree walking. This keeps symbol selection declarative and easy to adjust.
- Build LSP `DocumentSymbol` results (not `SymbolInformation`) to represent hierarchy. `interface` symbols will collect `op_dcl` children during a single traversal over query captures.
- Map IDL declaration kinds to LSP `SymbolKind` (e.g., `Struct`, `Enum`, `Interface`, `Method`, `EnumMember/TypeParameter` as appropriate for `bitmask`), choosing the closest LSP enum rather than inventing new kinds.
- Use the current full parse tree for the file; no incremental update path is added.

## Risks / Trade-offs

- [Risk] Query coverage may miss edge cases in grammar (aliases or alternative productions) → Mitigation: keep the query focused on the canonical grammar nodes and extend later if needed.
- [Risk] Incorrect hierarchy if `op_dcl` is captured without reliable parent linkage → Mitigation: derive parent interface from capture metadata or tree ancestry at extraction time.
- [Trade-off] Non-incremental approach may be slower on large files → acceptable for now given scope and simplicity.
