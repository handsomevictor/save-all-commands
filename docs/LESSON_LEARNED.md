# 开发过程 Bug 记录

## [2026-03-23] BrowseItem 未实现 Clone trait 导致 clippy warning

**现象**：手写了 `impl BrowseItem { pub fn clone(...) }` 方法，cargo clippy 报告 `should_implement_trait` warning：方法名 `clone` 与标准 trait `std::clone::Clone::clone` 混淆。

**原因**：Rust 要求若方法名与标准 trait 方法同名，应直接 `derive` 或手动 `impl` 该 trait，而不是在 inherent impl 里定义同名方法。

**解决**：为 `BrowseItem` 添加 `#[derive(Clone)]`，删除手写的 `clone` 方法。

---

## [2026-03-23] Style 类型实现了 Copy，不应调用 clone()

**现象**：`meta_style.clone()` 触发 clippy `clone_on_copy` warning。

**原因**：ratatui 的 `Style` 类型实现了 `Copy` trait，直接赋值即可复制，无需显式 `.clone()`。

**解决**：移除多余的 `.clone()` 调用。
