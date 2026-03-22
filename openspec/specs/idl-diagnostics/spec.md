## ADDED Requirements

### Requirement: Publish diagnostics for parse errors

The language server SHALL publish diagnostics for all tree-sitter `ERROR` nodes
found in an IDL document's current parse tree.

#### Scenario: Document contains parse errors

- **WHEN** the server parses an IDL document containing syntax errors
- **THEN** it publishes diagnostics covering each `ERROR` node range

### Requirement: Diagnostics are produced on document changes

The server MUST regenerate diagnostics from the latest full parse tree whenever
the document is opened or changed.

#### Scenario: Document edited after errors

- **WHEN** the client edits an IDL document and the server reparses it
- **THEN** the published diagnostics reflect the current set of `ERROR` nodes
