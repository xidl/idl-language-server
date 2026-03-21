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

### Tokyo Night Semantic Highlighting

Tokyo Night themes define only a small set of semantic token colors. If you want richer semantic
highlighting, add the following to your `settings.json`:

```json
{
  "editor.semanticTokenColorCustomizations": {
    "[Tokyo Night]": {
      "rules": {
        "namespace": "#7aa2f7",
        "type": "#7aa2f7",
        "class": "#7aa2f7",
        "struct": "#7aa2f7",
        "interface": "#7aa2f7",
        "enum": "#7aa2f7",
        "typeParameter": "#bb9af7",
        "parameter.declaration": "#e0af68",
        "parameter": "#d9d4cd",
        "variable.declaration": "#bb9af7",
        "variable": "#c0caf5",
        "property.declaration": "#73daca",
        "property": "#c0caf5",
        "enumMember": "#ff9e64",
        "event": "#ff9e64",
        "function": "#bb9af7",
        "method": "#bb9af7",
        "macro": "#f7768e",
        "keyword": "#f7768e",
        "modifier": "#7dcfff",
        "comment": "#565f89",
        "string": "#9ece6a",
        "number": "#ff9e64",
        "regexp": "#b4f9f8",
        "operator": "#89ddff",
        "decorator": "#7dcfff",
        "*.defaultLibrary": "#2ac3de",
        "variable.defaultLibrary": "#2ac3de",
        "property.defaultLibrary": "#2ac3de",
        "function.defaultLibrary": "#2ac3de",
        "method.defaultLibrary": "#2ac3de"
      }
    },
    "[Tokyo Night Storm]": {
      "rules": {
        "namespace": "#7aa2f7",
        "type": "#7aa2f7",
        "class": "#7aa2f7",
        "struct": "#7aa2f7",
        "interface": "#7aa2f7",
        "enum": "#7aa2f7",
        "typeParameter": "#bb9af7",
        "parameter": "#e0af68",
        "variable": "#c0caf5",
        "property": "#c0caf5",
        "enumMember": "#ff9e64",
        "event": "#ff9e64",
        "function": "#bb9af7",
        "method": "#bb9af7",
        "macro": "#f7768e",
        "keyword": "#f7768e",
        "modifier": "#7dcfff",
        "comment": "#565f89",
        "string": "#9ece6a",
        "number": "#ff9e64",
        "regexp": "#b4f9f8",
        "operator": "#89ddff",
        "decorator": "#7dcfff",
        "*.declaration": "#7aa2f7",
        "variable.defaultLibrary": "#7dcfff",
        "property.defaultLibrary": "#7dcfff",
        "function.defaultLibrary": "#7dcfff",
        "method.defaultLibrary": "#7dcfff"
      }
    },
    "[Tokyo Night Moon]": {
      "rules": {
        "namespace": "#82aaff",
        "type": "#82aaff",
        "class": "#82aaff",
        "struct": "#82aaff",
        "interface": "#82aaff",
        "enum": "#82aaff",
        "typeParameter": "#c099ff",
        "parameter": "#ffc777",
        "variable": "#c8d3f5",
        "property": "#c8d3f5",
        "enumMember": "#ff966c",
        "event": "#ff966c",
        "function": "#c099ff",
        "method": "#c099ff",
        "macro": "#ff757f",
        "keyword": "#ff757f",
        "modifier": "#7fdbca",
        "comment": "#636da6",
        "string": "#c3e88d",
        "number": "#ff966c",
        "regexp": "#b4f9f8",
        "operator": "#86e1fc",
        "decorator": "#7fdbca",
        "*.declaration": "#82aaff",
        "variable.defaultLibrary": "#7fdbca",
        "property.defaultLibrary": "#7fdbca",
        "function.defaultLibrary": "#7fdbca",
        "method.defaultLibrary": "#7fdbca"
      }
    },
    "[Tokyo Night Light]": {
      "rules": {
        "namespace": "#2e7de9",
        "type": "#2e7de9",
        "class": "#2e7de9",
        "struct": "#2e7de9",
        "interface": "#2e7de9",
        "enum": "#2e7de9",
        "typeParameter": "#9854f1",
        "parameter": "#8c6c3e",
        "variable": "#343b58",
        "property": "#343b58",
        "enumMember": "#b15c00",
        "event": "#b15c00",
        "function": "#9854f1",
        "method": "#9854f1",
        "macro": "#f52a65",
        "keyword": "#f52a65",
        "modifier": "#007197",
        "comment": "#848cb5",
        "string": "#587539",
        "number": "#b15c00",
        "regexp": "#007197",
        "operator": "#006a83",
        "decorator": "#007197",
        "*.declaration": "#2e7de9",
        "variable.defaultLibrary": "#007197",
        "property.defaultLibrary": "#007197",
        "function.defaultLibrary": "#007197",
        "method.defaultLibrary": "#007197"
      }
    }
  }
}
```

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
