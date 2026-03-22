# save-all-commands (sac) — Claude Code 开发指令

## 项目概述

使用 Rust 开发一个终端命令管理工具，二进制名称为 `sac`。用户可以将常用的 shell 命令按层级分类存储，通过 TUI 界面快速检索并将命令注入到终端输入框。唯一的查找入口是 `sac`（无参数），直接进入 TUI 界面。

---

## 强制文档规范

项目根目录下 `docs/` 目录必须维护以下文档，**每次修改代码后必须同步更新，不得遗漏**：

### `docs/README.md`
项目介绍、安装方式、快速上手，面向用户。

### `docs/PROGRESS.md`
每次更新记录，格式：
```
## v0.x.x — YYYY-MM-DD
### 新增
### 修复
### 变更
```

### `docs/STRUCTURE.md`
完整项目结构，每个文件的职责说明，每次新增/删除文件后更新。

### `docs/LESSON_LEARNED.md`
开发过程中遇到的 bug、原因分析、解决方案。格式：
```
## [日期] Bug 标题
**现象**：
**原因**：
**解决**：
```

### `docs/TUTORIAL.md`
所有支持的功能和交互方式的完整说明，面向用户，每次新增功能后更新。

### `docs/COMMANDS.md`
从 TUTORIAL 提取的纯命令列表，用于本地测试 copy 用，格式：
```
# 基础命令
sac
sac add
sac add --folder <folder-id>
...
```

---

## 代码规范

- 所有文档使用**中文**
- 代码中的 comment 使用**英文**
- Rust edition 2021
- 错误处理：统一使用 `anyhow`
- 序列化：`serde` + `toml`
- 禁止使用 `unwrap()`，一律用 `?` 或 `anyhow::bail!`

---

## 技术栈

```toml
[package]
name = "sac"
version = "0.1.0"
edition = "2021"

[dependencies]
clap = { version = "4", features = ["derive"] }
ratatui = "0.28"
crossterm = "0.28"
serde = { version = "1", features = ["derive"] }
toml = "0.8"
nucleo-matcher = "0.3"
arboard = "3"
dirs = "5"
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1"
reqwest = { version = "0.12", features = ["blocking"] }

[dev-dependencies]
tempfile = "3"
```

---

## 数据模型

### 配置文件：`~/.sac/config.toml`

```toml
# sac 主配置文件

[general]
auto_check_remote = true     # 每天首次运行自动检查远端更新
last_check = "2025-01-01"    # 上次检查日期，程序自动维护

[commands_source]
mode = "local"               # "local" 或 "remote"
path = "~/.sac/commands.toml"  # mode = local 时使用
url = ""                     # mode = remote 时使用，支持 gist raw url / s3 / 任意公开 http

[shell]
type = "zsh"                 # "zsh" / "bash" / "fish"
```

### 命令存储文件：`~/.sac/commands.toml`

两张扁平表，用 `parent` 字段表达层级关系，人工维护直观清晰：

```toml
[[folders]]
id = "personal"
parent = ""
name = "Personal"

[[folders]]
id = "personal.devops"
parent = "personal"
name = "DevOps"

[[folders]]
id = "personal.devops.k8s"
parent = "personal.devops"
name = "Kubernetes"

[[folders]]
id = "personal.s3"
parent = "personal"
name = "S3"

[[folders]]
id = "work"
parent = ""
name = "Work"

[[commands]]
id = 1
folder = "personal.devops.k8s"
cmd = "kubectl get pods -n {namespace}"
desc = "列出指定 namespace 的 pods"
comment = "需要配置好 kubectl context"
tags = ["k8s", "devops"]
last_used = ""

[[commands]]
id = 2
folder = "personal.s3"
cmd = "aws s3 ls s3://{bucket}"
desc = "列出 bucket 内容"
comment = ""
tags = ["aws", "s3"]
last_used = ""
```

### Rust 数据结构

```rust
// src/store.rs

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Folder {
    pub id: String,       // dot-separated path, e.g. "personal.devops.k8s"
    pub parent: String,   // parent folder id, "" for root
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Command {
    pub id: u32,
    pub folder: String,
    pub cmd: String,
    pub desc: String,
    #[serde(default)]
    pub comment: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub last_used: String,   // ISO 8601 timestamp or empty string
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Store {
    #[serde(default)]
    pub folders: Vec<Folder>,
    #[serde(default)]
    pub commands: Vec<Command>,
}
```

---

## 层级约束

- 每个 folder 下，**子 folder 最多 10 个**（编号 1-9，第 10 个用 0）
- 每个 folder 下，**直接 command 最多 10 个**（编号 1-9，第 10 个用 0）
- folder 编号和 command 编号**各自独立**，互不影响
- `Store::validate()` 在每次写入时强制检查，超出则返回具体错误信息

---

## TUI 设计

### 唯一入口

```
sac
```

无任何参数，直接进入 TUI。没有 `sac search`、`sac list` 等查找类子命令，所有查找交互均在 TUI 内完成。

### 初始界面（浏览模式）

```
┌─ sac ────────────────────────────────────────────────────────┐
│  [数字] 直接选择  [↑↓] 移动  [Enter] 确认  [q/ESC] 返回/退出 │
├──────────────────────────────────────────────────────────────┤
│  🔍                                            模糊搜索       │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  📁  [1]  Personal                                           │
│  📁  [2]  Work                                               │
│  📁  [3]  DevOps                                             │
│  ──────────────────────────────────────────────────────      │
│  $   [1]  docker ps -a                                       │
│           列出所有容器                                        │
│  $   [2]  git status                                         │
│           查看工作区状态                                      │
│                                                              │
│  3 folders · 2 commands                                      │
└──────────────────────────────────────────────────────────────┘
```

**设计要点**：
- 搜索框默认就在顶部，光标始终在搜索框
- 用户不需要按任何键"激活"搜索，直接打字即触发模糊搜索
- 直接按数字键即跳转对应编号，不需要先移开光标
- folder 和 command 分区显示，中间有分隔线
- folder 编号和 command 编号各自从 1 开始，第 10 个用 0

### 搜索框有内容时（模糊搜索）

```
┌─ sac ────────────────────────────────────────────────────────┐
│  [数字] 直接选择  [↑↓] 移动  [Enter] 确认  [ESC] 清空搜索    │
├──────────────────────────────────────────────────────────────┤
│  🔍 kube                                       模糊搜索       │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  $  [1]  kubectl get pods -n {namespace}                     │
│          📁 Personal > DevOps > Kubernetes                   │
│          列出指定 namespace 的 pods                           │
│                                                              │
│  $  [2]  kubectl logs -f {pod} -n {namespace}                │
│          📁 Personal > DevOps > Kubernetes                   │
│          实时查看 pod 日志                                    │
│                                                              │
│  $  [3]  kubectl exec -it {pod} -- /bin/bash                 │
│          📁 Work > DevOps                                     │
│          进入容器 shell                                       │
│                                                              │
│  3 个结果                                                     │
└──────────────────────────────────────────────────────────────┘
```

搜索时跨所有 folder 展示结果，每条结果显示面包屑路径。

### 精确搜索模式（输入 `//` 前缀）

```
┌─ sac ────────────────────────────────────────────────────────┐
│  [数字] 直接选择  [↑↓] 移动  [Enter] 确认  [ESC] 清空搜索    │
├──────────────────────────────────────────────────────────────┤
│  🔍 //kubectl                                  精确搜索       │
├──────────────────────────────────────────────────────────────┤
│  （只显示完整包含 "kubectl" 字符串的结果）                     │
└──────────────────────────────────────────────────────────────┘
```

- 搜索框以 `//` 开头时自动切换为精确搜索模式，右上角标注变化
- 删除第二个 `/` 后自动退回模糊搜索模式
- `//` 后为空时等同于模糊搜索框为空，显示全部内容

---

## 键位完整映射

### 搜索框为空时

| 按键 | 行为 |
|------|------|
| `1`-`9` | 立即跳转对应编号（folder 则进入，command 则选中退出 TUI） |
| `0` | 跳转第 10 个条目 |
| `↑` `↓` | 移动光标高亮 |
| `Enter` | 确认高亮项（folder 进入，command 选中退出） |
| `q` / `ESC` | 有父级则返回上一层；在根目录则退出程序 |
| 任意字母/符号 | 字符追加到搜索框，立即触发模糊搜索，切换到搜索结果视图 |

### 搜索框有内容时

| 按键 | 行为 |
|------|------|
| 继续输入字符 | 追加到搜索框，实时更新结果 |
| `1`-`9`, `0` | 立即选中对应编号的搜索结果，退出 TUI 输出命令 |
| `↑` `↓` | 移动结果列表光标 |
| `Enter` | 确认当前高亮的结果，退出 TUI 输出命令 |
| `Backspace` | 删除最后一个字符（按 char 删，正确处理中文多字节），结果实时更新 |
| `ESC` | 清空搜索框，回到浏览模式，显示当前 folder 内容 |

### 搜索模式切换规则

| 搜索框内容 | 模式 | 右上角标注 |
|-----------|------|-----------|
| 空 | 浏览模式 | （无） |
| 任意非 `//` 开头内容 | 模糊搜索 | `模糊搜索` |
| `//` + 任意内容 | 精确搜索 | `精确搜索` |
| 仅 `//` | 精确搜索，结果为全部 | `精确搜索` |

---

## 搜索结果排序规则

```
第一优先级：cmd 字段精确包含 query（大小写不敏感）
第二优先级：desc 字段精确包含 query
第三优先级：模糊匹配综合评分
            cmd 权重 × 3
            desc 权重 × 2
            comment 权重 × 1
            tags 权重 × 1
第四优先级（同分时）：last_used 越近排越前
第五优先级（从未使用时）：按 command id 升序
```

搜索 haystack 构建方式：
```rust
format!("{} {} {} {}", cmd.cmd, cmd.desc, cmd.comment, cmd.tags.join(" "))
```

中文无需分词，`nucleo-matcher` 按字符跳跃匹配天然支持中文。

---

## CLI 子命令（管理类，非查找类）

```
sac                              # 进入 TUI（唯一查找入口）

sac add                          # 交互式添加命令（会提示输入 folder、cmd、desc 等）
sac add --folder <folder-id>     # 在指定 folder 下添加命令

sac new-folder <name>                        # 在根目录新建 folder
sac new-folder <name> --parent <folder-id>   # 在指定 folder 下新建子 folder

sac edit <command-id>            # 交互式编辑指定 id 的命令
sac delete <command-id>          # 删除指定 id 的命令（二次确认）

sac sync                         # 手动触发远端同步检查
sac sync --force                 # 强制用远端覆盖本地（二次确认）

sac config                       # 显示当前完整配置内容
sac config set <key> <value>     # 修改配置项，例：sac config set general.auto_check_remote false

sac where config                 # 显示配置文件完整路径
sac where commands               # 显示命令存储文件完整路径

sac install                      # 自动检测 shell 类型，写入 shell integration 到对应 rc 文件

sac export <path>                # 导出 commands.toml 到指定路径
sac import <path>                # 从文件导入，显示 diff 后 prompt 用户确认再写入
```

---

## 功能模块详细说明

### `src/main.rs`
程序入口。解析 CLI 参数。无参数时启动 TUI。有子命令时执行对应逻辑。每日首次运行时（对比 `last_check` 与今日日期）若 `auto_check_remote = true` 则触发远端同步检查。TUI 退出后若 `app.output` 有值则打印到 stdout（供 shell function 捕获）。

### `src/cli.rs`
使用 clap derive 宏定义所有子命令和参数。

### `src/config.rs`
读写 `~/.sac/config.toml`。提供：
- `Config::load()` — 读取配置，文件不存在时自动创建默认配置和 `~/.sac/` 目录
- `Config::save()` — 写回配置文件
- `Config::set(key, value)` — 按点分路径设置配置项

### `src/store.rs`
读写 `~/.sac/commands.toml`。提供：
- `Store::load()` / `Store::load_from(path)` — 读取，不存在时返回空 Store
- `Store::save()` / `Store::save_to(path)` — 写回
- `Store::children_folders(parent_id)` — 获取直接子 folder 列表
- `Store::folder_commands(folder_id)` — 获取 folder 下的直接 command 列表
- `Store::breadcrumb(folder_id)` — 返回从根到该 folder 的名称列表，用于显示路径
- `Store::next_command_id()` — 返回当前最大 command id + 1
- `Store::validate()` — 检查所有约束，返回 `Result<()>`

### `src/search.rs`
搜索逻辑。提供：
- `SearchResult` 结构体：包含 `command: Command`、`score: u32`、`folder_path: Vec<String>`
- `Searcher::new()` — 初始化 nucleo matcher
- `Searcher::fuzzy_search(query, store)` — 模糊搜索，返回按分数排序的结果列表
- `Searcher::exact_search(query, store)` — 精确搜索，返回包含 query 的结果列表
- 内部方法 `build_haystack(cmd)` — 拼接所有可搜索字段
- 内部方法 `weighted_score(pattern, cmd)` — 分字段加权评分

### `src/sync.rs`
远端同步逻辑。提供：
- `check_network()` — 尝试连接 `1.1.1.1:80`，超时 2 秒，返回 `bool`
- `fetch_remote(url)` — HTTP GET 下载内容，返回字符串
- `parse_remote(content)` — 解析下载内容为 `Store`，格式不合法时返回错误
- `diff_stores(local, remote)` — 对比两个 Store，返回新增、修改、冲突的 command id 列表
- `sync_check(config, store)` — 完整流程：检查网络 → 下载 → 解析 → 显示 diff → prompt 用户 y/n → 写入本地

### `src/tui/mod.rs`
TUI 模块入口，re-export `App` 和 `run_tui`。

### `src/tui/app.rs`
TUI 状态机：

```rust
pub enum Mode {
    Browse,   // browsing folder tree, search box is empty
    Search,   // search box has content, showing filtered results
}

pub enum SearchMode {
    Fuzzy,    // default
    Exact,    // triggered by "//" prefix
}

pub enum BrowseItem {
    Folder(Folder),
    Command(Command),
}

pub struct App {
    // mode
    pub mode: Mode,
    pub search_mode: SearchMode,

    // browse state
    pub current_folder: String,        // current folder id, "" = root
    pub items: Vec<BrowseItem>,        // items shown in current folder
    pub selected_index: usize,         // cursor position in items list
    pub breadcrumb: Vec<String>,       // folder names from root to current

    // search state
    pub search_query: String,          // raw text in search box (includes "//" prefix if any)
    pub search_results: Vec<SearchResult>,
    pub search_selected: usize,        // cursor in search results

    // output
    pub output: Option<String>,        // set when user confirms a command
    pub should_quit: bool,

    // data
    pub store: Store,
    pub searcher: Searcher,
}
```

关键方法：
- `App::new(store)` — 初始化，加载根目录内容
- `App::handle_key(key_event)` — 主键盘事件分发
- `App::handle_key_browse(key)` — 浏览模式按键处理
- `App::handle_key_search(key)` — 搜索模式按键处理
- `App::enter_folder(folder_id)` — 进入子 folder，更新 items 和 breadcrumb
- `App::go_back()` — 返回上一层，更新状态
- `App::confirm_command(cmd)` — 设置 output，更新 last_used，保存 store
- `App::refresh_search()` — 根据 search_query 和 search_mode 调用对应搜索方法更新 search_results
- `App::update_search_mode()` — 检查 search_query 是否以 `//` 开头，自动切换 SearchMode

### `src/tui/ui.rs`
ratatui 渲染逻辑：
- `render(frame, app)` — 主渲染函数，根据 mode 调用对应渲染函数
- `render_header(frame, area, app)` — 顶部键位提示栏
- `render_search_box(frame, area, app)` — 搜索框，显示 query 和搜索模式标注
- `render_browse(frame, area, app)` — 浏览模式：分区显示 folder 和 command，高亮当前选中项
- `render_search_results(frame, area, app)` — 搜索结果：每条显示编号、cmd、面包屑、desc
- `render_status_bar(frame, area, app)` — 底部状态栏，显示数量信息

### `src/shell.rs`
Shell integration：
- `detect_shell()` — 读取 `$SHELL` 环境变量判断 shell 类型
- `get_rc_path(shell_type)` — 返回对应 rc 文件路径
- `write_integration(shell_type)` — 向 rc 文件追加 function，写入前检查是否已存在，避免重复写入

---

## Shell Integration 模板

`sac install` 命令自动检测 shell 类型并将以下对应内容追加到 rc 文件。

### Zsh（追加到 `~/.zshrc`）

```bash
# sac shell integration — do not edit this block
function sac() {
  local result
  result=$(command sac "$@" 2>/dev/tty)
  if [[ -n "$result" ]]; then
    BUFFER="$result"
    zle redisplay
  fi
}
# end sac shell integration
```

### Bash（追加到 `~/.bashrc`）

```bash
# sac shell integration — do not edit this block
function sac() {
  local result
  result=$(command sac "$@" 2>/dev/tty)
  if [[ -n "$result" ]]; then
    READLINE_LINE="$result"
    READLINE_POINT=${#READLINE_LINE}
  fi
}
# end sac shell integration
```

### Fish（追加到 `~/.config/fish/config.fish`）

```fish
# sac shell integration — do not edit this block
function sac
  set result (command sac $argv 2>/dev/tty)
  if test -n "$result"
    commandline $result
  end
end
# end sac shell integration
```

`write_integration()` 在写入前检查文件中是否已包含 `sac shell integration`，存在则跳过，避免重复追加。

---

## 每日首次运行检查逻辑

```
启动时：
  读取 config.last_check
  若 last_check != 今日日期 且 auto_check_remote = true：
    调用 sync_check()
    无网络时静默跳过，不报错，不影响启动
    有网络时显示检查结果，需用户输入 y 确认才写入
  更新 config.last_check = 今日日期
  保存 config
```

---

## 测试规范

**每个功能必须有两个 test case：**
1. **正常用例**：验证 expected output
2. **错误用例**：验证错误输入时返回有意义的错误信息（不能 panic，错误信息不能为空）

### 测试文件结构

```
tests/
├── test_store.rs        # Store 读写、增删改查、validate
├── test_search.rs       # 模糊搜索、精确搜索、中文搜索、空 query、无结果
├── test_sync.rs         # diff 逻辑、冲突检测、格式校验
├── test_config.rs       # 配置读写、set 方法
├── test_cli.rs          # 所有子命令解析
└── test_validation.rs   # 约束检查（folder/command 数量上限）
```

### 测试示例

```rust
// tests/test_store.rs

#[cfg(test)]
mod tests {
    use tempfile::tempdir;
    use sac::store::{Command, Folder, Store};

    // Test: successfully load a valid commands.toml
    #[test]
    fn test_load_store_success() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("commands.toml");
        std::fs::write(
            &path,
            r#"
[[folders]]
id = "personal"
parent = ""
name = "Personal"

[[commands]]
id = 1
folder = "personal"
cmd = "echo hello"
desc = "say hello"
comment = ""
tags = []
last_used = ""
            "#,
        )
        .unwrap();

        let store = Store::load_from(&path).unwrap();
        assert_eq!(store.folders.len(), 1);
        assert_eq!(store.commands.len(), 1);
        assert_eq!(store.commands[0].cmd, "echo hello");
    }

    // Test: loading a malformed toml returns a descriptive error, not a panic
    #[test]
    fn test_load_store_malformed_toml() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("commands.toml");
        std::fs::write(&path, "this is ::: not valid toml").unwrap();

        let result = Store::load_from(&path);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(!err_msg.is_empty(), "error message must not be empty");
    }

    // Test: validate passes when folder children are within limit
    #[test]
    fn test_validate_folder_limit_ok() {
        let mut store = Store::default();
        store.folders.push(Folder {
            id: "root".into(),
            parent: "".into(),
            name: "Root".into(),
        });
        for i in 1..=9 {
            store.folders.push(Folder {
                id: format!("root.f{}", i),
                parent: "root".into(),
                name: format!("Folder {}", i),
            });
        }
        assert!(store.validate().is_ok());
    }

    // Test: validate returns a descriptive error when folder limit is exceeded
    #[test]
    fn test_validate_folder_limit_exceeded() {
        let mut store = Store::default();
        store.folders.push(Folder {
            id: "root".into(),
            parent: "".into(),
            name: "Root".into(),
        });
        for i in 1..=11 {
            store.folders.push(Folder {
                id: format!("root.f{}", i),
                parent: "root".into(),
                name: format!("Folder {}", i),
            });
        }
        let result = store.validate();
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("10") || msg.contains("最多") || msg.contains("limit"),
            "error message should mention the limit, got: {}",
            msg
        );
    }
}
```

```rust
// tests/test_search.rs

#[cfg(test)]
mod tests {
    use sac::search::Searcher;
    use sac::store::{Command, Folder, Store};

    fn make_store() -> Store {
        Store {
            folders: vec![Folder {
                id: "devops".into(),
                parent: "".into(),
                name: "DevOps".into(),
            }],
            commands: vec![
                Command {
                    id: 1,
                    folder: "devops".into(),
                    cmd: "kubectl get pods".into(),
                    desc: "列出所有 pods".into(),
                    comment: "".into(),
                    tags: vec!["k8s".into()],
                    last_used: "".into(),
                },
                Command {
                    id: 2,
                    folder: "devops".into(),
                    cmd: "docker ps -a".into(),
                    desc: "列出所有容器".into(),
                    comment: "".into(),
                    tags: vec![],
                    last_used: "".into(),
                },
            ],
        }
    }

    // Test: fuzzy search returns relevant results
    #[test]
    fn test_fuzzy_search_returns_results() {
        let store = make_store();
        let mut searcher = Searcher::new();
        let results = searcher.fuzzy_search("kubectl", &store);
        assert!(!results.is_empty());
        assert_eq!(results[0].command.id, 1);
    }

    // Test: fuzzy search with no matching query returns empty results
    #[test]
    fn test_fuzzy_search_no_match() {
        let store = make_store();
        let mut searcher = Searcher::new();
        let results = searcher.fuzzy_search("zzznomatch999", &store);
        assert!(results.is_empty());
    }

    // Test: exact search matches chinese description
    #[test]
    fn test_exact_search_chinese() {
        let store = make_store();
        let searcher = Searcher::new();
        let results = searcher.exact_search("列出所有", &store);
        assert_eq!(results.len(), 2);
    }

    // Test: exact search with wrong case still matches (case-insensitive)
    #[test]
    fn test_exact_search_case_insensitive() {
        let store = make_store();
        let searcher = Searcher::new();
        let results = searcher.exact_search("KUBECTL", &store);
        assert!(!results.is_empty());
        assert_eq!(results[0].command.id, 1);
    }
}
```

---

## 开发顺序

请严格按以下顺序实现，每完成一个阶段必须更新所有文档后再进入下一阶段：

### 阶段一：数据层
1. 项目初始化，`Cargo.toml`，目录结构
2. `src/store.rs` — 数据结构定义、所有读写和查询方法、`validate()`
3. `src/config.rs` — 配置读写、自动初始化
4. `tests/test_store.rs`、`tests/test_config.rs`、`tests/test_validation.rs`
5. `cargo test` 全部通过
6. 更新所有文档，外加 `.gitignore`

### 阶段二：搜索层
1. `src/search.rs` — Searcher、fuzzy_search、exact_search、加权评分
2. `tests/test_search.rs` — 英文、中文、空 query、无结果、大小写不敏感
3. `cargo test` 全部通过
4. 更新所有文档

### 阶段三：TUI 层
1. `src/tui/app.rs` — App 状态机、所有键盘事件处理、Mode 切换
2. `src/tui/ui.rs` — ratatui 渲染（浏览模式 + 搜索结果模式）
3. 手动启动验证 TUI 交互流程
4. 更新所有文档

### 阶段四：CLI 与入口
1. `src/cli.rs` — 所有子命令定义
2. `src/main.rs` — 入口逻辑、每日检查调用
3. `tests/test_cli.rs`
4. `cargo test` 全部通过
5. 更新所有文档

### 阶段五：同步层
1. `src/sync.rs` — 网络检测、下载、解析、diff、prompt
2. `tests/test_sync.rs`
3. `cargo test` 全部通过
4. 更新所有文档

### 阶段六：Shell Integration
1. `src/shell.rs` — shell 检测、rc 文件路径、写入逻辑（去重）
2. `sac install` 命令接入
3. 手动验证 zsh/bash 注入效果
4. 更新所有文档

### 阶段七：收尾
1. `cargo test` 全部通过
2. `cargo clippy` 无 warning
3. 所有文档终态更新
4. `docs/COMMANDS.md` 完整命令列表终态确认

---

## 约束清单（开发全程持续遵守）

- [ ] 禁止 `unwrap()`，全部用 `?` 或 `anyhow::bail!`
- [ ] 每个 folder 下子 folder 最多 10 个（1-9 和 0），command 最多 10 个，`validate()` 强制检查
- [ ] 数字键 1-9 和 0 直接映射选择，无需 Enter 等待
- [ ] 搜索框默认激活，无需额外按键触发
- [ ] `//` 前缀切换精确搜索，删掉第二个 `/` 自动退回模糊搜索
- [ ] 选中命令后只输出到 stdout，不自动执行命令
- [ ] 没有网络时跳过远端检查，静默跳过，不影响正常启动
- [ ] 远端更新必须用户输入 `y` 确认才写入本地
- [ ] 所有错误信息必须有意义，不能是空字符串或 generic panic
- [ ] `cargo test` 全部通过才能进入下一阶段
- [ ] 每个功能两个 test case：正常用例 + 错误用例
- [ ] 每次代码变更后必须同步更新全部六个文档