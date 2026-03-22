## ADDED Requirements

### Requirement: Preserve externally visible LSP behavior
The server MUST preserve existing LSP request/response semantics and capabilities while refactoring internal structure.

#### Scenario: Refactor does not change LSP behavior
- **WHEN** the server is refactored into modules and context-based state management
- **THEN** all LSP capabilities, commands, and response shapes remain unchanged
