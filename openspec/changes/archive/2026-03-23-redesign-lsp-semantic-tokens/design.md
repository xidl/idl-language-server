## Context

The extension currently emits semantic tokens with a lossy mapping that
collapses specific tokens into a flag token. This reduces fidelity and makes
highlighting inconsistent across editors. The extension’s `package.json`
semantic token legend also diverges from rust-analyzer’s commonly used token
types and modifiers, leading to mismatches between server output and client
configuration.

## Goals / Non-Goals

**Goals:**

- Preserve original token types in LSP semantic token output instead of
  collapsing to a flag token.
- Align the semantic token legend (types and modifiers) with rust-analyzer’s VS
  Code extension.
- Update this extension’s `package.json` contributions to match the new legend
  so clients can render correctly.

**Non-Goals:**

- Reworking non-semantic-token features such as diagnostics, code actions, or
  hover.
- Re-theming or changing editor color themes; this change only updates token
  identity and legend.

## Decisions

- **Adopt rust-analyzer’s token types and modifiers as the canonical legend.**
  - Rationale: maximizes interoperability and minimizes surprises for users who
    rely on existing themes.
  - Alternatives: keep current custom legend; partially align. Rejected because
    it perpetuates mismatch and requires ongoing maintenance.
- **Stop mapping tokens to a generic flag token and preserve original token
  types.**
  - Rationale: provides richer semantic information and reduces ambiguity in
    highlighting.
  - Alternatives: map only certain categories or keep a fallback token. Rejected
    because it still loses detail and complicates debugging.
- **Update extension `package.json` to match the server legend.**
  - Rationale: ensures the client recognizes all emitted token types/modifiers.
  - Alternatives: keep client config unchanged and remap server output. Rejected
    because it preserves the existing drift.

## Risks / Trade-offs

- [Risk] Editors/themes that relied on the old token mapping may see different
  colors. → Mitigation: align with rust-analyzer legend that is widely
  supported.
- [Risk] Partial implementation could create a server/client mismatch. →
  Mitigation: update server legend and `package.json` in the same change and add
  validation/tests if present.
