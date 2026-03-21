# IDL Language Server

IDL language server and VS Code extension (`vscode-idl-language`).

## Features

- Semantic tokens
- Inlay hints
- Diagnostics
- Code completion
- Go to definition
- Find references
- Rename
- Format

## VS Code Extension

The extension bundles a platform-specific `idl-language-server` binary on release.
If you need to override the server path locally, set `IDL_LANGUAGE_SERVER_PATH`.

## Development

Install dependencies:

```bash
pnpm install
```

Build extension bundle:

```bash
pnpm run compile
```

Watch mode:

```bash
pnpm run watch
```
