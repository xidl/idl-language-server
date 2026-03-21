## 1. Rename Core

- [x] 1.1 Implement rename symbol collection using existing definition + references logic
- [x] 1.2 Build `WorkspaceEdit` with all matching ranges updated to the new name
- [x] 1.3 Return no edits when cursor is not on a renameable symbol

## 2. LSP Integration

- [x] 2.1 Wire `textDocument/rename` handler and register capability
- [x] 2.2 Ensure rename uses current document content (no incremental cache)

## 3. Tests

- [x] 3.1 Add/update tests for rename on definitions and references
- [x] 3.2 Add/update tests for non-symbol rename behavior (if test harness exists)
