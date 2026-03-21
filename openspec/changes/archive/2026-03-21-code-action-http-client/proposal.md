## Why

The language server does not currently provide a code action to start an HTTP client workflow for XIDL services. Developers must manually generate OpenAPI docs and run a viewer, which slows iteration. Adding a single code action tied to HTTP-related IDL content makes the workflow discoverable and one-click.

## What Changes

- Add a new code action on `interface` declarations that starts or stops an HTTP client preview based on server state.
- Detect HTTP-relevant documents by presence of `docs/rfc/http.md`-related annotations or `pragma xidlc service`.
- Invoke `xidlc` to generate OpenAPI, then run a Redoc server using Axum + utoipa-scalar with vendored assets.
- Regenerate OpenAPI output when the source document changes.
- When the preview is running, expose the scalar UI URL via hover action.

## Capabilities

### New Capabilities
- `idl-code-action-http-client`: Provide a code action on interfaces to start/stop an HTTP client preview when HTTP-related IDL content is present.

### Modified Capabilities
- None.

## Impact

- LSP code action handling and server state for start/stop.
- New background process orchestration for xidlc and the Redoc server.
- Additional Rust modules for HTTP preview server and vendored scalar assets.
- Integration with `docs/rfc/http.md` annotation detection and `pragma xidlc service` parsing.
