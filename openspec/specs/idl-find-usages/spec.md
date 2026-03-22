## ADDED Requirements

### Requirement: Find usages from definitions

The language server SHALL resolve `textDocument/references` requests by
returning locations of `declaration` captures that match the symbol name of a
`definition` capture at the cursor.

#### Scenario: Find usages from a definition

- **WHEN** the client requests `textDocument/references` on a symbol captured as
  `definition`
- **THEN** the server returns locations of matching `declaration` captures in
  the document

### Requirement: Do not return references for declarations

The server MUST return no references when the cursor is on a `declaration`
capture; only definitions can search for usages.

#### Scenario: Find usages from a declaration

- **WHEN** the client requests `textDocument/references` on a symbol captured as
  `declaration`
- **THEN** the server returns no result
