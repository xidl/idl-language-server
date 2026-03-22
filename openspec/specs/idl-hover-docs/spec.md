## ADDED Requirements

### Requirement: Provide hover content for query-marked nodes

The language server SHALL implement `textDocument/hover` by matching
query-marked nodes and selecting a documentation template associated with the
capture (e.g., `@http` -> `http.md`).

#### Scenario: Hover over a marked annotation

- **WHEN** the client hovers a node captured by the hover query
- **THEN** the server resolves the mapped template and returns rendered hover
  content

### Requirement: Render templates with embedded assets

The server MUST embed documentation templates using `rust-embed` and render them
at hover time with `minijinja`.

#### Scenario: Template rendering

- **WHEN** a hover template is resolved
- **THEN** the server loads it from embedded assets and renders it with
  `minijinja`

### Requirement: Provide rich rendering context

The server MUST provide a rendering context that includes:

- document name and path
- symbol context (xidl HIR)
- symbol lookup and reference discovery helpers (file, line, column)
- curated external references (e.g., MDN) as template-provided links or excerpts

#### Scenario: Template uses context helpers

- **WHEN** a template accesses context or helper functions
- **THEN** the server provides the requested symbol and reference data

### Requirement: Fail safely when template missing

The server SHOULD return a safe fallback hover message when a template is
missing or rendering fails.

#### Scenario: Missing template

- **WHEN** a mapped template cannot be found or fails to render
- **THEN** the server returns a fallback hover message
