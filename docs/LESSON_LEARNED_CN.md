# 开发过程 Bug 记录

## [2026-03-23] 致命：/dev/tty 写入后 stdin 仍被 ZLE 持有导致 event::read() 永久阻塞

**现象**：改用 `/dev/tty` 作为渲染后端后，TUI 画面能正常显示，但按任意键均无响应，终端完全卡死。

**原因**：zsh 的 ZLE（Zsh Line Editor）在交互式 shell 中始终持有 stdin（fd 0）的控制权。在 `$()` 子 shell 里，子进程虽然继承了 stdin，但 ZLE 仍通过进程组和终端所有权拦截键盘输入。`crossterm::event::read()` 从 stdin 读取事件，由于 ZLE 的拦截，这个读取永远不会返回，表现为完全卡死。改用 `/dev/tty` 作为写入后端只解决了渲染问题，没有解决读取问题。

**解决**：参照 fzf 的实现方式：以 `O_RDWR` 打开 `/dev/tty`，然后通过 `libc::dup2(tty_fd, STDIN_FILENO)` 将 stdin 重定向到 `/dev/tty`。dup2 之后，fd 0 直接指向终端设备文件，不再通过 ZLE 管理的文件描述符路径，`event::read()` 从新的 stdin（即 `/dev/tty`）读取键盘事件，正常响应。

```rust
let tty_file = OpenOptions::new().read(true).write(true).open("/dev/tty")?;
unsafe { libc::dup2(tty_file.as_raw_fd(), libc::STDIN_FILENO) };
enable_raw_mode()?;
let backend = CrosstermBackend::new(BufWriter::new(tty_file));
```

**教训**：在 zsh 交互式 shell 中，`$()` 子进程的 stdin 表面上继承自 shell，但 ZLE 会拦截键盘输入。任何需要在此上下文中读取键盘的 TUI 工具，必须通过 `dup2` 将 stdin 重映射到 `/dev/tty`，而不是假设 stdin 可直接使用。这是 fzf、peco 等所有主流 TUI 选择器的标准实践。

---

## [2026-03-23] 致命：TUI 使用 stderr 后终端卡死

**现象**：将渲染后端从 stdout 改为 stderr 后，运行 `sac` 终端直接卡死，无任何输出，只能强制 Ctrl+C。

**原因**：`crossterm::terminal::size()` 在 Unix 上通过 `ioctl(STDOUT_FILENO, TIOCGWINSZ, ...)` 获取终端尺寸，stdout 优先。Shell integration 执行 `result=$(command sac "$@" 2>/dev/tty)` 时，stdout 是 pipe 而非 TTY，`TIOCGWINSZ` 返回 ENOTTY。`Terminal::new(backend)?` 在尺寸查询失败时出错返回，但此时 `enable_raw_mode()` 已经调用，cleanup 代码（`disable_raw_mode` / `LeaveAlternateScreen`）在错误传播路径上可能未执行，终端被锁在 raw mode + alternate screen，表现为完全卡死。

**解决**：直接打开 `/dev/tty` 作为渲染后端（`OpenOptions::new().write(true).open("/dev/tty")`）。`/dev/tty` 始终指向进程的控制终端，不受任何 stdout/stderr 重定向影响，是 fzf、vim、tmux 等工具的行业标准做法。同时将 cleanup 全部改为 `let _ =`，确保即使中间步骤失败，终端状态也一定被恢复。

**教训**：stdout/stderr 在 shell 调用链中随时可能被重定向。TUI 工具必须打开 `/dev/tty` 作为渲染目标，不能依赖 stdout/stderr 的当前状态。cleanup 代码必须全部用 `let _ =` 包裹，任何 `?` 都可能跳过后续的终端恢复操作。

---

## [2026-03-23] 设计缺陷：TUI folder/command 分区编号导致 [1] 含义歧义

**现象**：TUI 中 folder 和 command 各自独立编号（folder [1]、command [1] 同时存在），按数字键时行为取决于当前 items 排列，违反"编号不可重复"的直觉认知，用户需要记住"先按哪个 1"。

**原因**：设计沿用了旧规范中"folder 编号和 command 编号各自独立"的设定，导致同一屏幕内数字 1-9 对应两套不同事物。

**解决**：取消分区 header 和分隔线，folder 和 command 在同一列表中按位置统一编号（1、2、3 …）。folder 仍排在 command 前面，但编号连续不重复。层级约束也同步改为合并上限（子 folder + command 合计最多 10 个）。

---

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

## [2026-03-23] 致命：dup2 重定向 stdin 后 crossterm "Failed to initialize input reader"

**现象**：通过 `dup2(tty_fd, STDIN_FILENO)` 将 stdin 重映射到 `/dev/tty` 后，crossterm 报错 `Failed to initialize input reader`，TUI 无法启动。

**原因**：crossterm 在 Unix 上使用 mio + kqueue/epoll 注册 stdin（fd 0）进行事件监听。dup2 执行后，fd 0 指向 `/dev/tty` 设备文件；但 kqueue 对终端设备文件的 EVFILT_READ 注册会失败（某些 macOS/BSD 版本不支持对 `/dev/tty` fd 的 kqueue 监听），导致 crossterm 内部 mio 初始化报错。

**解决**：放弃 dup2 方案。根本修复在于改变 shell integration 架构：从 `result=$(command sac ...)` 改为 `command sac >"$tmp" 2>/dev/tty`（tmpfile 方案）。新方案中 sac 进程运行在前台，不在 `$()` 子 shell 内，ZLE 不再拦截 stdin，stdin 直接继承自交互式 shell（已是真实 TTY），crossterm 正常注册 kqueue 事件，无需 dup2。TUI 渲染后端只需 write-only 打开 `/dev/tty` 即可。

**教训**：kqueue/epoll 对 `/dev/tty` fd 的可注册性在 macOS 上不稳定。不要用 dup2 强制替换 stdin；正确解法是确保进程从一开始就在正确的上下文（前台、真实 TTY stdin）中运行，这需要在 shell integration 层面解决而非在 Rust 层面打补丁。

---

## [2026-03-23] 致命：$() 捕获方案根本缺陷 + ZLE widget 上下文错误

**现象**：`sac install` 后，每次 `source ~/.zshrc` 或直接运行 `sac --version` 都报 `sac:zle: widgets can only be called when ZLE is active`。

**原因**：旧的 shell integration 无条件调用 `zle redisplay` 和设置 `BUFFER`，但这些操作只能在 ZLE widget 上下文内（即用户正在编辑命令行时）调用。`source ~/.zshrc` 和直接命令调用均不在 ZLE 上下文中，导致报错。此外，`result=$(command sac ...)` 将 sac 放入 `$()` 子 shell，ZLE 通过进程组拦截 stdin，`event::read()` 永久阻塞。

**解决**：完全重写 shell integration，采用 tmpfile 方案：
- `[[ $# -eq 0 ]]` 门控：只有无参调用才启动 TUI，否则直接 `command sac "$@"` passthrough（修复 `--version` 等子命令被捕获的问题）
- `command sac >"$tmp" 2>/dev/tty`：sac 在前台运行，stdout 重定向到 tmpfile，无 `$()` 包裹，ZLE 不拦截
- `if zle; then ... else print -z -- "$result"; fi`：检测是否在 ZLE 上下文，否则用 `print -z` 将命令放入 next-prompt buffer

**教训**：`$()` + ZLE 是两个独立的致命问题。shell integration 的正确架构：(1) 用参数数量门控 TUI 入口；(2) 用 tmpfile 替代 `$()`；(3) 用 `if zle` 检测上下文后再调用 ZLE builtins。

---

## [2026-03-25] 致命：$(<file) + emulate -L zsh 在 ZLE 上下文中吞掉 \<换行>

**现象**：包含 `\<换行>`（行延续语法，例如 `curl -X POST "..." \`）的命令，在 TUI 中选中后粘贴到终端，反斜杠全部丢失，多行命令变成了一行且没有 `\`。

**原因**：zsh snippet 中存在两个叠加问题：
1. `result=$(<"$tmp")` — 即使在 `emulate -L zsh` 下，zsh 在命令替换期间仍会将 `\<换行>` 视为行延续序列，静默地去掉反斜杠并将前后两行合并。
2. `BUFFER=$result` + `zle redisplay` — 在某些 zsh 版本中，直接向 `BUFFER` 赋值并调用 `zle redisplay` 可能触发 ZLE 对字符串内容的二次解析，造成额外的转义处理。

**解决**：组合应用四项修改：
- `{ IFS='' read -r -d '' result; } < "$tmp"` — `read -r` 在 shell 层面完全禁用反斜杠转义处理；`read -d ''` 读取直到 EOF 而不是换行符。这是 zsh/bash 中二进制安全读取文件内容的标准模式。
- `LBUFFER=$result; RBUFFER=''` — 通过 `LBUFFER`/`RBUFFER`（编辑缓冲区的左右两部分）赋值，而不是直接操作 `BUFFER`，绕过 BUFFER 级别的 ZLE 再解析。
- `zle reset-prompt` 替代 `zle redisplay` — `reset-prompt` 只重绘 prompt 行，不触发 ZLE 对 buffer 内容的再次处理。
- `print -rz` 替代 `print -z`（非 ZLE 路径）— `-r` 标志禁用 `print` 的转义处理，与 `read -r` 读取端的保证保持一致。
- 移除 `emulate -L zsh` — 该选项并非必要，且会创建一个与字符串处理产生不可预测交互的副作用环境。

```zsh
# 修复前（错误）
result=$(<"$tmp")
BUFFER=$result
CURSOR=${#BUFFER}
zle redisplay

# 修复后
{ IFS='' read -r -d '' result; } < "$tmp"; result=${result%$'\n'}
LBUFFER=$result
RBUFFER=''
zle reset-prompt
```

**教训**：任何读取文件内容并设置 ZLE buffer 的 shell 函数：
- 必须使用 `read -r`，防止反斜杠被解释。`$(<file)` 对任意内容并不安全。
- 优先使用 `LBUFFER`/`RBUFFER` 而非直接操作 `BUFFER`，避免 ZLE 再解析副作用。
- 视觉刷新应使用 `zle reset-prompt` 而非 `zle redisplay`。
- 向 history/next-prompt buffer 写入内容时使用 `print -rz`（不加 `-r` 不安全）。
- 用包含 `\`、`\\`、`\n`、`\ `（反斜杠+空格）的命令进行测试——这些字符最容易被 shell 字符串处理静默篡改。

---

## [2026-03-23] Style 类型实现了 Copy，不应调用 clone()

**现象**：`meta_style.clone()` 触发 clippy `clone_on_copy` warning。

**原因**：ratatui 的 `Style` 类型实现了 `Copy` trait，直接赋值即可复制，无需显式 `.clone()`。

**解决**：移除多余的 `.clone()` 调用。
