# 开发过程 Bug 记录

## [2026-03-23] 致命：TUI 使用 stdout 导致选中命令被直接执行

**现象**：配置好 shell integration 后，在 TUI 中选中一条命令，命令没有出现在终端输入栏，而是被直接执行了。

**原因**：Shell integration 的工作原理是 `result=$(command sac "$@" 2>/dev/tty)`——用子 shell 捕获 `sac` 的 stdout，再将结果写入 `BUFFER`。
TUI 使用 `io::stdout()` 作为 ratatui 的渲染后端，导致所有 UI 转义序列（`EnterAlternateScreen`、光标控制、颜色码等）全部流入 stdout，被 `$(...)` 一并捕获。
最终 `BUFFER` 被设置为 `<大量转义码> + 命令文本`，zsh 在 redisplay 后识别 BUFFER 内容并触发执行，命令被运行。

**解决**：将 TUI 后端改为 `io::stderr()`：

```rust
// src/tui/mod.rs — 修复前
let mut stdout = io::stdout();
execute!(stdout, EnterAlternateScreen)?;
let backend = CrosstermBackend::new(stdout);

// 修复后
let mut stderr = io::stderr();
execute!(stderr, EnterAlternateScreen)?;
let backend = CrosstermBackend::new(stderr);
```

Shell integration 已通过 `2>/dev/tty` 将 stderr 重定向到真实终端，TUI 渲染正常显示；stdout 只剩最终的纯命令文本，`BUFFER` 得到干净的命令字符串，不再触发执行。

**教训**：任何通过 `$(...)` 捕获输出的 CLI 工具，若内部启动 TUI，必须将 TUI 渲染绑定到 stderr 或直接打开 `/dev/tty`，而非 stdout。stdout 必须保持干净，只输出最终机器可读结果。

---

## [2026-03-23] BrowseItem 未实现 Clone trait 导致 clippy warning

**现象**：手写了 `impl BrowseItem { pub fn clone(...) }` 方法，cargo clippy 报告 `should_implement_trait` warning：方法名 `clone` 与标准 trait `std::clone::Clone::clone` 混淆。

**原因**：Rust 要求若方法名与标准 trait 方法同名，应直接 `derive` 或手动 `impl` 该 trait，而不是在 inherent impl 里定义同名方法。

**解决**：为 `BrowseItem` 添加 `#[derive(Clone)]`，删除手写的 `clone` 方法。

---

## [2026-03-23] Style 类型实现了 Copy，不应调用 clone()

**现象**：`meta_style.clone()` 触发 clippy `clone_on_copy` warning。

**原因**：ratatui 的 `Style` 类型实现了 `Copy` trait，直接赋值即可复制，无需显式 `.clone()`。

**解决**：移除多余的 `.clone()` 调用。
