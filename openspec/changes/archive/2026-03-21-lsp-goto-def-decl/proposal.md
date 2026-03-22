## Why

The IDL language server lacks go-to navigation, making it hard to jump between
symbol definitions, declarations, and usages. Adding explicit go-to
definition/declaration behavior improves developer navigation and aligns with
LSP expectations.

## What Changes

- Add go-to definition and go-to declaration handling based on tree-sitter
  queries.
- Treat definitions as `definition` and type usages as `declaration` for
  matching symbols.
- Allow go-to only from declarations to corresponding definitions; definitions
  only support finding usages.

## Capabilities

### New Capabilities

- `idl-goto-definition`: Resolve definition locations for symbols referenced as
  declarations.
- `idl-goto-declaration`: Resolve declaration locations for type usages (as
  declaration sites) and allow jumping to definitions.
- `idl-find-usages`: Resolve usage locations for symbols defined in the file.

### Modified Capabilities

- None.

## Impact

- LSP request handling for `textDocument/definition`,
  `textDocument/declaration`, and `textDocument/references`.
- tree-sitter-idl query definitions for definitions and declarations.
- Symbol matching logic based on captured names.
