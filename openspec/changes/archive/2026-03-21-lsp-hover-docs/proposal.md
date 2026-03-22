## Why

The IDL language server currently lacks rich hover documentation tied to
annotations and symbols. Adding hover rendering based on embedded templates
improves discoverability and provides consistent, reusable docs in-editor.

## What Changes

- Implement `textDocument/hover` using tree-sitter queries to map nodes (e.g.,
  `@http`) to documentation templates.
- Render hover content via embedded Jinja templates (minijinja) with
  Rust-provided context and helper functions.
- Embed documentation assets with `rust-embed` and dynamically resolve templates
  by name/path.
- Allow templates to include curated external references (for example MDN links
  or short excerpts).

## Capabilities

### New Capabilities

- `idl-hover-docs`: Provide hover documentation rendered from embedded Jinja
  templates with symbol and reference context.

### Modified Capabilities

- None.

## Impact

- LSP hover handler and capability registration.
- New embedded documentation templates and rendering pipeline using minijinja.
- Integration with xidl HIR and reference lookup utilities for hover context.
