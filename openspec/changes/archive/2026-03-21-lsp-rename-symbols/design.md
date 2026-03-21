## Context

The IDL language server already supports go-to definition, declaration, and references using tree-sitter queries and name matching within a document. Rename can build on those capabilities by collecting all matching occurrences and returning a `WorkspaceEdit` that replaces them with a new name.

## Goals / Non-Goals

**Goals:**
- Implement `textDocument/rename` for IDL files.
- Rename all occurrences that match the symbol name (definition + references) within the same document.
- Reuse existing go-to definition and references logic for matching.

**Non-Goals:**
- Cross-file or workspace-wide rename.
- Semantic validation beyond simple name matching.
- Incremental or background indexing.

## Decisions

- Use the existing goto-symbol query and matching logic to gather definition and reference ranges, then produce a `WorkspaceEdit` with `TextEdit` replacements.
- Only allow rename when the cursor is on a renameable symbol (definition or declaration capture). If not, return no edit.
- Keep rename scope limited to the current document to match the current symbol resolution approach.

## Risks / Trade-offs

- [Risk] Name-based matching can rename unrelated symbols with the same name in different scopes → Mitigation: initial scope is same document; refine with scope-aware matching later.
- [Risk] Edits may overlap if captures are nested → Mitigation: dedupe ranges and apply consistent ordering.
- [Trade-off] No cross-file rename → acceptable for initial feature scope.
