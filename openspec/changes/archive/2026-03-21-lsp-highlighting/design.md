## Context

当前 LSP 缺少语法高亮能力，编辑器只能依赖基础文本渲染与诊断信息。`xidlc` 已有诊断高亮实现（`xidlc/src/diagnostic/highlight.rs`），可复用其 token 分类与 style 映射思路。目标是在 LSP 中引入 `tree-sitter-idl` + `tree-sitter-highlight`，并采用全量刷新策略（文档变更后重新计算全量高亮），避免增量复杂度。

## Goals / Non-Goals

**Goals:**
- 在 LSP 侧实现 IDL 语法高亮输出，基于 tree-sitter 的高亮查询。
- 映射/对齐 `xidlc` 的高亮 token 分类，保证诊断高亮一致性。
- 文档变动时全量重算高亮，确保实现简单可靠。
- 提供可通过 LSP/HTTP（curl）触发的验证方式，并在指定示例文件上确认效果。
- 编译无 warning。

**Non-Goals:**
- 不实现增量高亮或增量语法树更新。
- 不改变已有诊断逻辑与语义分析规则。
- 不在客户端侧实现渲染，只提供 LSP 语义 token 数据。

## Decisions

- 采用 `tree-sitter-idl` 解析 + `tree-sitter-highlight` 生成高亮范围：
  - 复用现有生态与高亮查询，减少自定义解析逻辑。
  - 与 `xidlc` 高亮类别做映射，保持 token 语义一致。
- 全量高亮策略：
  - 每次 `didOpen` / `didChange` 触发时重新解析全文并生成完整 tokens。
  - 省去增量逻辑与状态同步复杂度，符合当前需求与规模。
- LSP 输出使用语义高亮（Semantic Tokens）而非自定义扩展：
  - 标准化，便于用 curl 或编辑器直接验证。
  - 与现有 LSP 协议兼容。

## Risks / Trade-offs

- [性能] 全量重算在大文件上可能增加延迟 → 先接受，后续如有需要再引入增量或缓存。
- [分类差异] `tree-sitter-highlight` 捕获的 token 与 `xidlc` 诊断分类可能存在偏差 → 在映射层做显式规则与回归验证。
- [依赖] 新增 tree-sitter 依赖可能引入编译特性或构建变更 → 明确 Cargo 依赖与特性配置，确保无 warning。
