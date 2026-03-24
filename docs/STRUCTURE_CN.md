# 项目结构说明

```
save-all-commands/
├── Cargo.toml              # 项目依赖配置
├── commands.toml.example   # 内置示例命令库（约 100 条命令）
├── src/
│   ├── lib.rs              # 库入口，导出所有模块
│   ├── main.rs             # 二进制入口，CLI 解析与子命令分发
│   ├── cli.rs              # clap derive 宏定义所有子命令和参数
│   ├── config.rs           # 配置文件读写（~/.sac/config.toml）
│   ├── store.rs            # 命令存储读写（~/.sac/commands.toml）
│   ├── search.rs           # 搜索逻辑（模糊搜索 + 精确搜索 + 加权评分）
│   ├── sync.rs             # 远端同步（HTTP 下载、diff、用户确认写入）
│   ├── shell.rs            # Shell Integration（zsh/bash/fish）
│   └── tui/
│       ├── mod.rs          # TUI 模块入口，run_tui 函数
│       ├── app.rs          # App 状态机（Mode、BrowseItem、键盘事件处理）
│       └── ui.rs           # ratatui 渲染逻辑（布局、表格、搜索框、状态栏）
├── tests/
│   ├── test_store.rs       # Store 读写、查询、validate 测试
│   ├── test_config.rs      # Config 读写、set 方法测试
│   ├── test_validation.rs  # 约束检查（folder/command 数量上限）测试
│   ├── test_search.rs      # 模糊搜索、精确搜索、中文、空 query 测试
│   ├── test_cli.rs         # CLI 子命令解析测试
│   └── test_sync.rs        # diff 逻辑、格式校验测试
└── docs/
    ├── README_CN.md        # 中文 README
    ├── PROGRESS.md         # 更新记录（英文）
    ├── PROGRESS_CN.md      # 更新记录（中文）
    ├── STRUCTURE.md        # 项目结构说明（英文）
    ├── STRUCTURE_CN.md     # 项目结构说明（中文，本文件）
    ├── LESSON_LEARNED.md   # 开发过程 Bug 记录（英文）
    ├── LESSON_LEARNED_CN.md # 开发过程 Bug 记录（中文）
    ├── TUTORIAL.md         # 完整功能说明（英文）
    ├── TUTORIAL_CN.md      # 完整功能说明（中文）
    ├── COMMANDS.md         # CLI 命令速查（英文）
    ├── COMMANDS_CN.md      # CLI 命令速查（中文）
    └── assets/
        └── screenshot.png  # TUI 界面截图
```

---

## 模块说明

### `src/main.rs`

二进制入口。解析命令行参数，根据子命令分发到对应的处理函数。无子命令时启动 TUI。

### `src/cli.rs`

使用 clap derive 宏定义所有子命令结构体和参数，提供类型安全的 CLI 解析，并自动生成 `--help` 文档。

### `src/config.rs`

负责读写 `~/.sac/config.toml`。提供 `Config` 结构体，支持通过键值路径（如 `general.auto_check_remote`）读取和修改配置项。

### `src/store.rs`

负责读写 `~/.sac/commands.toml`。`Store` 结构体保存扁平的 `Folder` 和 `Command` 列表，提供以下方法：
- 树遍历（`children_folders`、`folder_commands`、`breadcrumb`）
- 结构校验（每个 folder 合计最多 10 个子项）
- 自动修复重复 command ID（`auto_fix_ids`）

### `src/search.rs`

通过 `Searcher` 实现两种搜索模式：

- **模糊搜索**（`fuzzy_search`）— 使用 nucleo-matcher 对 `cmd`、`desc`、`comment`、`tags` 进行加权模糊评分。优先级从高到低：tag 匹配 → cmd 精确包含 → desc 精确包含 → comment 精确包含 → 模糊评分。同分时按 `last_used` 降序，再按 ID 升序。
- **精确搜索**（`exact_search`）— 由 `//` 前缀触发，返回拼合 haystack 中包含查询词子字符串的命令。

### `src/sync.rs`

通过 HTTP GET 获取远端 TOML 文件，解析为 `Store`，与本地数据进行 diff 比较，在终端展示新增/修改/删除的命令，用户确认后写入。支持 `--force` 跳过确认。

### `src/shell.rs`

为 zsh、bash、fish 生成并安装 shell 集成脚本：
1. 用参数数量门控 TUI 入口——子命令直接 passthrough
2. 用 tmpfile 方案代替 `$()`，避免 ZLE 拦截 stdin
3. 读取 tmpfile 并将结果写入 shell 输入缓冲区（zsh/bash 用 `BUFFER`，fish 用 `commandline`）
4. 用 `if zle` 检测 ZLE 上下文，避免在非 ZLE 环境调用 ZLE builtins

`write_integration()` 能识别新旧两种 snippet 格式，自动执行原地升级。

### `src/tui/mod.rs`

TUI 入口。以 write-only 模式打开 `/dev/tty` 作为 ratatui 后端（绕过 shell 调用链中对 stdout/stderr 的任何重定向）。运行主事件循环：渲染 → 读取键盘事件 → 更新 App 状态 → 循环。退出时通过 `let _ =` 包裹的 cleanup 调用恢复终端状态。

### `src/tui/app.rs`

`App` 结构体是核心状态机，包含：
- `mode: Mode` — `Browse` 或 `Search`
- `search_mode: SearchMode` — `Fuzzy` 或 `Exact`
- `current_folder`、`breadcrumb` — 导航状态
- `items: Vec<BrowseItem>` — 当前 folder 的子项（folder + command）
- `search_query`、`search_results`、`search_selected` — 搜索状态
- `output: Option<String>` — 退出时填入终端的命令文本

核心方法：`handle_key()`、`enter_folder()`、`go_back()`、`load_items()`、`confirm_command()`、`refresh_search()`、`effective_query()`。

### `src/tui/ui.rs`

使用 ratatui 渲染 TUI。布局（从上到下）：

1. **Header**（1 行）— 键位提示栏
2. **Search box**（3 行）— 查询输入框 + 模式标签
3. **Main panel**（填满剩余空间）— 浏览表格或搜索结果表格
4. **Status bar**（1 行）— 项目数量统计或搜索统计

两种表格均采用三列布局：

| 列 | 宽度 | 内容 |
|----|------|------|
| 编号 | 6 字符 | `[1]`–`[0]`，超出 10 个后留空 |
| 命令 | 约 52% 可用宽度，限定 20–52 | `$  <cmd>`，超长截断为 `…` |
| 描述 | 剩余宽度 | 自动换行 desc（最多 3 行）+ 搜索模式下的 meta 行 |
