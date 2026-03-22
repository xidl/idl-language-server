## ADDED Requirements

### Requirement: Resolve declaration locations from type usages

The language server SHALL resolve `textDocument/declaration` requests by
locating matching `declaration` symbols for the symbol at the cursor when it
represents a type usage.

#### Scenario: Go-to declaration from a type usage

- **WHEN** the client requests `textDocument/declaration` on a symbol captured
  as `declaration`
- **THEN** the server returns the location of that `declaration` capture

### Requirement: Allow declaration to jump to definitions

The server MUST allow `textDocument/declaration` results to be used for go-to
definition resolution, matching declaration symbols to their definitions by
name.

#### Scenario: Declaration resolves to definition

- **WHEN** a declaration symbol name matches a definition symbol name
- **THEN** the server can resolve go-to definition from that declaration
