# 项目结构说明

```
save-all-commands/
├── Cargo.toml              # 项目依赖配置
├── src/
│   ├── lib.rs              # 库入口，导出所有模块
│   ├── main.rs             # 二进制入口，CLI 解析与子命令分发
│   ├── cli.rs              # clap derive 宏定义所有子命令和参数
│   ├── config.rs           # 配置文件读写（~/.sac/config.toml）
│   ├── store.rs            # 命令存储读写（~/.sac/commands.toml）
│   ├── search.rs           # 搜索逻辑（模糊搜索 + 精确搜索 + 加权评分）
│   ├── sync.rs             # 远端同步（HTTP 下载、diff、用户确认）
│   ├── shell.rs            # Shell Integration（zsh/bash/fish）
│   └── tui/
│       ├── mod.rs          # TUI 模块入口，run_tui 函数
│       ├── app.rs          # App 状态机（Mode、BrowseItem、键盘事件处理）
│       └── ui.rs           # ratatui 渲染逻辑
├── tests/
│   ├── test_store.rs       # Store 读写、查询、validate 测试
│   ├── test_config.rs      # Config 读写、set 方法测试
│   ├── test_validation.rs  # 约束检查（folder/command 数量上限）测试
│   ├── test_search.rs      # 模糊搜索、精确搜索、中文、空 query 测试
│   ├── test_cli.rs         # CLI 子命令解析测试
│   └── test_sync.rs        # diff 逻辑、格式校验测试
└── docs/
    ├── README.md           # 项目介绍、安装、快速上手（本文件）
    ├── PROGRESS.md         # 版本更新记录
    ├── STRUCTURE.md        # 项目结构说明（本文件）
    ├── LESSON_LEARNED.md   # 开发过程 bug 记录
    ├── TUTORIAL.md         # 完整功能使用说明
    └── COMMANDS.md         # 纯命令列表（本地测试用）
```

---

## 模块说明

### `src/main.rs`

二进制入口。解析命令行参数，根据子命令分发到对应的处理函数。无子命令时启动 TUI。

### `src/cli.rs`

使用 clap derive 宏定义所有子命令结构体和参数，提供类型安全的 CLI 解析。

### `src/config.rs`

负责读写 `~/.sac/config.toml`。提供 `Config` 结构体，支持通过键值路径（如 `general.auto_check_remote`）读取和修改配置项。

### `src/store.rs`

负责读写 `~/.sac/commands.toml`。提供 `Store` 结构体，管理 folder 树和命令列表，支持增删改查及结构校验。

### `src/search.rs`

实现两种搜索模式：
- **模糊搜索**：使用 nucleo-matcher 对命令名称和描述进行加权评分排序
- **精确搜索**：当 query 以 `//` 开头时，对命令内容进行精确子字符串匹配

### `src/sync.rs`

通过 HTTP 从远端 URL 下载命令数据，与本地数据进行 diff 比较，向用户展示差异，确认后写入本地。

### `src/shell.rs`

生成 zsh、bash、fish 的 shell 集成脚本，并写入对应的 shell 配置文件。集成后，TUI 选中命令时会通过特定机制将命令填入终端输入栏。

### `src/tui/`

基于 ratatui 构建的 TUI 界面：
- `app.rs`：维护应用状态机，处理键盘事件，在浏览模式和搜索模式之间切换
- `ui.rs`：根据当前状态渲染界面布局、命令列表、搜索框等组件
