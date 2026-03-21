## ADDED Requirements

### Requirement: Provide folding ranges for interface and module blocks
The language server SHALL return folding ranges for `interface` and `module` declarations based on tree-sitter query captures.

#### Scenario: File contains interface and module blocks
- **WHEN** the client requests folding ranges for an IDL document
- **THEN** the server returns folding ranges covering each `interface` and `module` declaration

### Requirement: Folding ranges are derived from the current parse tree
The server MUST compute folding ranges from the latest full parse tree without incremental caches.

#### Scenario: Folding after edits
- **WHEN** an IDL document is edited and folding ranges are requested
- **THEN** the server computes folding ranges from the updated parse tree
