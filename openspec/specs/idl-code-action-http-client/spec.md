## ADDED Requirements

### Requirement: Provide interface code action for HTTP client preview
The language server SHALL offer a code action on `interface` declarations to start or stop an HTTP client preview when the document is HTTP-relevant.

#### Scenario: Action appears on interface in HTTP-relevant document
- **WHEN** the cursor is on an `interface` declaration and the document is HTTP-relevant
- **THEN** the server returns a code action named `start http client` or `stop http client` depending on current preview state

### Requirement: Detect HTTP-relevant documents
The server SHALL treat a document as HTTP-relevant when it contains any element tied to `docs/rfc/http.md` or a `pragma xidlc service` directive.

#### Scenario: Annotation-driven relevance
- **WHEN** the document contains any HTTP annotation defined by `docs/rfc/http.md`
- **THEN** the document is considered HTTP-relevant

#### Scenario: Pragma-driven relevance
- **WHEN** the document contains `pragma xidlc service`
- **THEN** the document is considered HTTP-relevant

### Requirement: Start preview by generating OpenAPI and serving via scalar
When starting the preview, the server MUST invoke `xidlc` to generate OpenAPI output and MUST start a Redoc-compatible server using Axum and utoipa-scalar with vendored assets.

#### Scenario: Start action
- **WHEN** the user selects `start http client`
- **THEN** the server generates OpenAPI with `xidlc` and starts the preview server

### Requirement: Stop preview when action is invoked again
When the preview is running, the server MUST stop the preview server when the user invokes `stop http client`.

#### Scenario: Stop action
- **WHEN** the user selects `stop http client`
- **THEN** the server stops the preview server for that document

### Requirement: Regenerate OpenAPI on document change
While a preview is running, the server MUST regenerate OpenAPI output when the source document changes.

#### Scenario: Document change triggers regeneration
- **WHEN** the document changes while the preview is running
- **THEN** the server regenerates the OpenAPI output used by the preview

### Requirement: Provide scalar URL via hover when preview is running
When the preview server is running for a document, the server MUST provide a hover action that opens the scalar UI URL.

#### Scenario: Hover provides scalar URL
- **WHEN** the user hovers an interface while the preview is running
- **THEN** the hover action includes a URL to the scalar UI for that preview
