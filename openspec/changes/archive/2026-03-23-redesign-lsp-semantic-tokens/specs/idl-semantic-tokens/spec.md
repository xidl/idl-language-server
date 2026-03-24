## ADDED Requirements

### Requirement: Semantic token legend matches rust-analyzer

The server SHALL expose a semantic token legend whose token types and modifiers
exactly match the ids defined in
`/Users/loongtao/rust-analyzer/editors/code/package.json` under
`semanticTokenTypes` and `semanticTokenModifiers`.

Token types: `angle`, `arithmetic`, `attribute`, `attributeBracket`, `bitwise`,
`boolean`, `brace`, `bracket`, `builtinAttribute`, `builtinType`, `character`,
`colon`, `comma`, `comparison`, `constParameter`, `const`, `derive`,
`deriveHelper`, `dot`, `escapeSequence`, `formatSpecifier`,
`invalidEscapeSequence`, `label`, `lifetime`, `logical`, `macroBang`,
`parenthesis`, `procMacro`, `punctuation`, `operator`, `selfKeyword`,
`selfTypeKeyword`, `semicolon`, `static`, `toolModule`, `typeAlias`, `union`,
`unresolvedReference`.

Token modifiers: `async`, `attribute`, `callable`, `constant`, `consuming`,
`controlFlow`, `crateRoot`, `injected`, `intraDocLink`, `library`, `macro`,
`mutable`, `procMacro`, `public`, `reference`, `trait`, `unsafe`.

#### Scenario: Legend uses the rust-analyzer token ids

- **WHEN** the client requests the semantic token legend
- **THEN** the returned token types and modifiers match the lists above without
  extra or missing ids

### Requirement: Preserve original token types

The server SHALL emit semantic tokens using their original token types and SHALL
NOT collapse distinct tokens into a generic flag token.

#### Scenario: Token types are preserved

- **WHEN** semantic tokens are computed for a document containing multiple token
  categories
- **THEN** each emitted token uses its original semantic token type rather than
  a single flag token

### Requirement: Extension legend matches server output

The extension `package.json` contributions SHALL declare `semanticTokenTypes`
and `semanticTokenModifiers` that match the server legend.

#### Scenario: Client configuration matches server legend

- **WHEN** the extension is loaded
- **THEN** its `semanticTokenTypes` and `semanticTokenModifiers` lists match the
  server legend ids
