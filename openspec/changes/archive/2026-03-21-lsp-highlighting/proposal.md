## Why

当前 LSP 缺少语法高亮能力，影响编辑体验与诊断可读性。通过引入 tree-sitter
高亮，可与现有诊断高亮实现对齐，并提供稳定的全量高亮输出。

## What Changes

- 新增基于 `tree-sitter-idl` + `tree-sitter-highlight` 的全量语法高亮管线。
- 复用/对齐 `xidlc` 诊断高亮实现逻辑，统一 token 映射与样式分类。
- LSP 在文档变动时重新全量高亮，不实现增量。
- 提供可通过 LSP/HTTP（curl）触发与验证高亮结果的流程。

## Capabilities

### New Capabilities

- `lsp-syntax-highlighting`: 为 IDL 文档提供基于 tree-sitter
  的语法高亮输出（全量刷新）。

### Modified Capabilities

- （无）

## Impact

- 影响 LSP 服务端高亮模块与文档变动处理路径。
- 新增或调整对 `tree-sitter-idl`、`tree-sitter-highlight` 的依赖与集成。
- 需要更新示例验证流程（启动 LSP、对指定 IDL 文件验证高亮）。
