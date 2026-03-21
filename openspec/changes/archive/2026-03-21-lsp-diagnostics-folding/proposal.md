## Why

The IDL language server does not surface parse error nodes as diagnostics, which makes syntax issues hard to locate. It also lacks code folding, so large interface or module blocks are harder to navigate.

## What Changes

- Emit diagnostics for all tree-sitter `ERROR` nodes in IDL documents.
- Add folding ranges for `interface` and `module` blocks using tree-sitter queries.
- Keep the implementation non-incremental and query-driven.

## Capabilities

### New Capabilities
- `idl-diagnostics`: Provide diagnostics for parse error nodes detected by tree-sitter.
- `idl-folding-ranges`: Provide folding ranges for `interface` and `module` declarations.

### Modified Capabilities
- None.

## Impact

- LSP diagnostics generation pipeline.
- tree-sitter-idl query definitions for diagnostics and folding ranges.
- LSP folding range handler integration and optional tests/fixtures.
