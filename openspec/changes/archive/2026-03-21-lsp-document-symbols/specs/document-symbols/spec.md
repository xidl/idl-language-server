## ADDED Requirements

### Requirement: Provide document symbols for IDL declarations
The language server SHALL respond to `textDocument/documentSymbol` for IDL documents by returning a list of symbols derived from the file's parse tree.

#### Scenario: Requesting document symbols for an IDL file
- **WHEN** the client sends `textDocument/documentSymbol` for an IDL document
- **THEN** the server returns DocumentSymbol entries derived from the current full parse tree

### Requirement: Capture specific declaration kinds via tree-sitter query
The server MUST use a tree-sitter query based on `tree-sitter-idl` to capture the following declaration nodes: `struct`, `enum`, `interface`, `bitmask`, and `op_dcl`.

#### Scenario: Query captures supported declaration kinds
- **WHEN** the server runs the document-symbol query on an IDL parse tree
- **THEN** it yields captures for each `struct`, `enum`, `interface`, `bitmask`, and `op_dcl` declaration in the file

### Requirement: Preserve interface hierarchy
The server MUST represent each `op_dcl` symbol as a child of its enclosing `interface` symbol.

#### Scenario: Interface with operations
- **WHEN** an IDL file contains an `interface` with one or more `op_dcl`
- **THEN** the returned DocumentSymbol for the interface includes those operations in its `children`

### Requirement: Non-incremental symbol generation
The server SHALL generate document symbols from the full parse tree and MUST NOT rely on incremental update state for this request.

#### Scenario: Request after edits
- **WHEN** the client requests document symbols after edits are applied
- **THEN** the server computes symbols from the latest full parse tree without requiring incremental symbol caches
