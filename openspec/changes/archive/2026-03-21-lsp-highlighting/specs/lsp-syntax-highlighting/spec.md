## ADDED Requirements

### Requirement: LSP provides full semantic tokens for IDL documents
The LSP server SHALL compute and return a complete semantic tokens set for an IDL document on open and on each content change.

#### Scenario: Open document
- **WHEN** the client sends `textDocument/didOpen` for an IDL file
- **THEN** the server returns a full semantic tokens payload for the entire document

#### Scenario: Document changed
- **WHEN** the client sends `textDocument/didChange` with updated content
- **THEN** the server recomputes and returns a full semantic tokens payload for the entire document

### Requirement: Highlighting uses tree-sitter-idl and tree-sitter-highlight
The server SHALL parse IDL source with `tree-sitter-idl` and produce highlight ranges with `tree-sitter-highlight`.

#### Scenario: Highlight generation
- **WHEN** the server receives a request to compute semantic tokens for IDL content
- **THEN** it uses tree-sitter-idl to parse and tree-sitter-highlight to produce highlight captures

### Requirement: Highlight categories map to LSP token types
The server SHALL map tree-sitter highlight captures to LSP semantic token types and modifiers, aligned with `xidlc` diagnostic highlight categories.

#### Scenario: Category mapping
- **WHEN** a highlight capture is produced
- **THEN** the server maps it to the configured LSP semantic token type/modifier and includes it in the tokens output

### Requirement: Full recomputation only (no incremental updates)
The server SHALL NOT require incremental semantic token updates; it SHALL only emit full tokens payloads.

#### Scenario: Client requests delta tokens
- **WHEN** the client requests semantic tokens delta
- **THEN** the server responds with a full tokens payload (or indicates delta is unsupported) according to its LSP capabilities
