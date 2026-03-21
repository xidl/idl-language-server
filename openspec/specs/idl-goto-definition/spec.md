## ADDED Requirements

### Requirement: Resolve definition locations from declarations
The language server SHALL resolve `textDocument/definition` requests by locating matching `definition` symbols for the symbol at the cursor when that symbol is a `declaration` capture.

#### Scenario: Go-to definition from a declaration
- **WHEN** the client requests `textDocument/definition` on a symbol captured as `declaration`
- **THEN** the server returns the location of the matching `definition` capture

### Requirement: Do not resolve definition from definition nodes
The server MUST NOT return a definition location when the cursor is on a `definition` capture; definitions only support finding usages.

#### Scenario: Go-to definition on a definition
- **WHEN** the client requests `textDocument/definition` on a symbol captured as `definition`
- **THEN** the server returns no result
