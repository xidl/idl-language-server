## 1. Inventory Current Behavior

- [x] 1.1 Locate server semantic token legend and mapping logic
- [x] 1.2 Identify current "flag token" mapping and where it is applied
- [x] 1.3 Record current extension `package.json` semantic token types/modifiers

## 2. Update Server Semantic Tokens

- [x] 2.1 Replace legend with rust-analyzer token types/modifiers list
- [x] 2.2 Update token mapping to preserve original token types (remove
      flag-token collapse)
- [x] 2.3 Ensure emitted tokens only use ids present in the updated legend

## 3. Update Extension Configuration

- [x] 3.1 Align `package.json` semanticTokenTypes with rust-analyzer list
- [x] 3.2 Align `package.json` semanticTokenModifiers with rust-analyzer list

## 4. Validation

- [x] 4.1 Verify semantic token legend in server matches `package.json`
- [x] 4.2 Run or add any existing semantic token tests/fixtures (if present)
