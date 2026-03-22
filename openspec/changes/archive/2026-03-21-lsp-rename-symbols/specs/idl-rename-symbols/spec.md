## ADDED Requirements

### Requirement: Rename all matching occurrences in a document

The language server SHALL handle `textDocument/rename` by returning edits that
replace all occurrences of the matched symbol name within the current document.

#### Scenario: Rename a symbol

- **WHEN** the client requests rename on a symbol captured as definition or
  declaration
- **THEN** the server returns a workspace edit that replaces all matching
  occurrences in the document

### Requirement: Use definition and references matching

The server MUST compute rename targets using the same symbol-matching logic as
go-to definition and find references.

#### Scenario: Rename uses references list

- **WHEN** the server computes rename edits
- **THEN** it includes the definition and all reference locations for the symbol

### Requirement: Reject rename on non-symbol positions

The server MUST return no edits when the cursor is not on a renameable symbol
capture.

#### Scenario: Rename on unrelated text

- **WHEN** the client requests rename on text that is not a symbol capture
- **THEN** the server returns no result
