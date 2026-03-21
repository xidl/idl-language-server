## Why

The IDL language server lacks rename support, so users must manually update all occurrences of a symbol. Adding rename improves refactoring ergonomics and builds on the recent go-to definition and references features.

## What Changes

- Implement `textDocument/rename` to update all matching symbol occurrences in a document.
- Use existing go-to definition and find references logic to compute all rename targets.
- Apply edits only when the cursor is on a renameable symbol.

## Capabilities

### New Capabilities
- `idl-rename-symbols`: Rename all occurrences of a symbol within a document using definition + references matching.

### Modified Capabilities
- None.

## Impact

- LSP rename request handling and edit generation.
- Symbol matching logic to map definitions and references for rename.
- Tests/fixtures for rename behavior (if test harness exists).
