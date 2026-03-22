## Context

The LSP server implementation is currently concentrated in `src/main.rs`, mixing
server wiring, document state, request handlers, and HTTP preview logic. This
makes ownership unclear and increases state passing and hard-coded strings. The
refactor should preserve public behavior while restructuring internals and
cleaning clippy warnings.

## Goals / Non-Goals

**Goals:**

- Split the LSP implementation into focused modules with clear ownership
  boundaries.
- Introduce a context/OO structure to centralize shared state and reduce ad-hoc
  parameter passing.
- Consolidate hard-coded strings and magic values into centralized constants.
- Resolve all existing `cargo clippy` warnings.

**Non-Goals:**

- No externally visible behavior changes to LSP capabilities or request
  semantics.
- No new user-facing features.
- No changes to the existing OpenSpec requirements set.

## Decisions

- **Module decomposition by responsibility.**
  - Rationale: keeps changes localized and makes ownership explicit; easier to
    reason about and test.
  - Alternative: keep in one file with regions; rejected because it does not
    reduce coupling.
- **Introduce an application context struct.**
  - Rationale: centralizes shared state (client, maps, caches) and reduces
    parameter plumbing.
  - Alternative: pass references across functions; rejected due to boilerplate
    and implicit dependencies.
- **Consolidate constants into a dedicated module.**
  - Rationale: enforces single source of truth for command names, strings, and
    config.
  - Alternative: leave inline literals; rejected due to drift and clippy
    warnings.
- **Clippy cleanup is part of refactor.**
  - Rationale: refactor will touch most of the code; fix warnings while
    structure is changing to avoid follow-up churn.

## Risks / Trade-offs

- **[Risk] Accidental behavior change** → Mitigation: keep request/response
  signatures unchanged and avoid spec-level changes; rely on existing usage
  patterns and minimal logic changes.
- **[Risk] Larger diff increases review cost** → Mitigation: commit in logical
  steps (module extraction, context introduction, constants, clippy fixes).
- **[Risk] Over-abstraction** → Mitigation: keep modules small and direct, avoid
  premature trait hierarchies.
