## Why

Semantic token output is currently lossy and inconsistent with widely used token
legends, making editor highlighting unreliable. Aligning with rust-analyzer’s
token legend and preserving original token types will improve accuracy and
interoperability now that semantic token usage is increasing in this extension.

## What Changes

- Redesign LSP semantic token mapping to preserve original token types instead
  of collapsing them into a flag token.
- Align the semantic token legend (token types and modifiers) with the set used
  by rust-analyzer’s VS Code extension.
- Update this extension’s `package.json` semantic token configuration to match
  the new legend.

## Capabilities

### New Capabilities

- `idl-semantic-tokens`: Provide LSP semantic tokens with a
  rust-analyzer-aligned legend and preserve original token types.

### Modified Capabilities

- (none)

## Impact

- Server-side semantic token generation and mapping logic.
- Extension contribution configuration in `package.json` (semantic token
  types/modifiers).
- Potential downstream editor theming and tests that assume previous token
  mapping.
