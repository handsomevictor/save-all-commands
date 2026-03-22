# 更新记录

## v0.1.2 — 2026-03-23

### 变更

- **TUI 统一编号**：移除 folder/command 分区 header 和分隔线，folder 和 command 混合排列，共享同一套 1-9/0 编号；选中 folder 则进入，选中 command 则填入终端
- **层级约束改为合并上限**：每个 folder 下子 folder + command **合计**最多 10 个（原为各自独立最多 10 个），与 TUI 键位一一对应
- **启动时自动修复重复 ID**：检测到 commands.toml 中存在重复 command id 时，自动按原有顺序重新分配 ID，打印警告后继续启动；结构性错误（超过 10 条限制）则打印错误信息后拒绝启动

### 新增

- `Store::auto_fix_ids()` 方法：检测并修复重复 command id，返回是否发生变更
- 新增 test case：`test_validate_combined_limit_ok/exceeded`、`test_auto_fix_ids_no_duplicates`、`test_auto_fix_ids_with_duplicates`、`test_auto_fix_ids_all_same`（共 46 个测试全部通过）

---

## v0.1.1 — 2026-03-23

### 修复

- **[致命] TUI 选中命令后直接执行而非填入输入栏**：将 TUI 后端从 `stdout` 改为 `stderr`，确保 shell integration 的 `result=$(command sac "$@" 2>/dev/tty)` 只捕获到纯命令文本，不再混入 TUI 转义码，彻底消除命令被意外执行的风险

---

## v0.1.0 — 2026-03-23

### 新增

- 数据层：Store（commands.toml 读写）、Config（config.toml 读写）
- 搜索层：模糊搜索（nucleo-matcher 加权评分）、精确搜索（// 前缀触发）
- TUI 层：浏览模式（树状 folder 导航）、搜索模式（实时过滤）
- CLI 子命令：add、new-folder、edit、delete、sync、config、where、install、export、import
- Shell Integration：zsh/bash/fish 三种 shell 支持，sac install 一键安装
- 同步层：远端 HTTP 同步、diff 展示、用户确认写入
- 完整测试套件：41 个测试用例，全部通过
