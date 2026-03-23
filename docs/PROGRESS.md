# 更新记录

## v0.1.5 — 2026-03-23

### 修复

- **[致命] `sac:zle: widgets can only be called when ZLE is active`**：完全重写 zsh/bash/fish shell integration
  - 用 `[[ $# -eq 0 ]]` 门控 TUI 入口，其他参数直接 `command sac "$@"` passthrough（修复 `--version`、`add` 等子命令被错误捕获的问题）
  - 用 tmpfile 方案（`command sac >"$tmp" 2>/dev/tty`）替代 `$()`，sac 进程运行在前台，ZLE 不再拦截 stdin
  - 用 `if zle; then ... else print -z -- "$result"; fi` 检测 ZLE 上下文，安全设置 BUFFER / 调用 `zle redisplay`
  - 新增 `# end sac shell integration` 结束标记；`sac install` 可检测旧格式并提示升级
- **[致命] `Error: Failed to initialize input reader`**：移除 `dup2(tty_fd, STDIN_FILENO)` — kqueue 无法监听通过 dup2 替换的 `/dev/tty` fd；新 shell integration 使 stdin 在运行时已是真实 TTY，不再需要 dup2
- **移除 `libc` 依赖**：dup2 方案废弃，`libc` crate 不再需要

---

## v0.1.4 — 2026-03-23

### 修复

- **`--version` 不可用**：clap `#[command]` 缺少 `version` 属性，现已补全，`sac --version` 正常输出版本号
- **[致命] TUI 卡死（ZLE 冲突）**：将 `/dev/tty` 改为 `O_RDWR` 打开，并通过 `dup2` 将 stdin（fd 0）重定向到 `/dev/tty`；zsh ZLE 在 `$()` 子 shell 中持有 stdin，导致 `event::read()` 永久阻塞，dup2 后 stdin 直接指向终端设备，绕过 ZLE 拦截
- **Ctrl+C 无法退出 TUI**：在浏览模式和搜索模式中均添加 `KeyModifiers::CONTROL + 'c'` 处理，按 Ctrl+C 立即退出并不输出任何命令

---

## v0.1.3 — 2026-03-23

### 修复

- **[致命] 运行 `sac` 后终端卡死**：将 TUI 渲染后端从 `stderr` 改为直接打开 `/dev/tty`（同时将 cleanup 全部改为 `let _ =` 确保任何路径下都能恢复终端状态）

---

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
