<div align="center">

# ⚡ sac — Save All Commands

**把你脑子里所有命令，装进一个 TUI**

[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange?logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Build](https://img.shields.io/badge/build-passing-brightgreen)](#)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux-lightgrey)](#)
[![Shell](https://img.shields.io/badge/shell-zsh%20%7C%20bash%20%7C%20fish-yellow)](#)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/handsomevictor/save-all-commands/pulls)

</div>

---

## 这是什么？它解决了什么问题？

作为开发者，你每天都在重复这些场景：

- 忘记了某个 `kubectl` 命令的完整写法，去翻文档或历史记录
- `docker run` 的参数太长，每次都要去翻之前的笔记或 Slack 消息
- 把命令存在 `~/.zshrc` alias 里，但 alias 太多又难以管理，更难搜索
- 命令分散在 Notion、备忘录、Notes、GitHub Gist，查找成本极高
- 团队共享命令只能靠 Confluence 或共享文档，没有统一的 CLI 工具

**`sac` 的核心理念**：把你所有常用命令放在一个地方，用一个 TUI 界面统一管理，按回车就能把命令填进终端输入栏，不执行、不搞破坏，你来决定要不要改参数再跑。

---

## 与现有工具的对比

| 功能 | shell alias | history / fzf | Notion/文档 | **sac** |
|------|:-----------:|:-------------:|:-----------:|:-------:|
| 保存带描述的命令 | 仅 alias 名 | 无 | 有 | ✅ |
| 按分类/层级组织 | ❌ | ❌ | 手动 | ✅ |
| 模糊搜索命令内容 | ❌ | 部分 | ❌ | ✅ |
| 填入终端但不执行 | ❌ | ❌ | ❌ | ✅ |
| 支持 `{placeholder}` 参数 | ❌ | ❌ | ❌ | ✅ |
| 跨 shell 支持 | 需手动配置 | 需手动配置 | ❌ | ✅ |
| 远端同步/团队共享 | ❌ | ❌ | 有 | ✅ |
| 纯文本，可 git 管理 | 勉强 | ❌ | ❌ | ✅ |
| 完全离线可用 | ✅ | ✅ | ❌ | ✅ |

> `fzf` + `history` 是个很好的工具，但它只能搜索你**用过**的命令。
> `sac` 帮你管理那些你**想用、偶尔用、但总忘的**命令。

---

## 安装

**前置要求**：Rust 工具链（推荐通过 [rustup](https://rustup.rs/) 安装）

```bash
git clone https://github.com/handsomevictor/save-all-commands.git
cd save-all-commands
cargo install --path .
```

安装完成后，配置 shell 集成（**强烈建议**，不配置则选中命令只会打印到 stdout）：

```bash
sac install
source ~/.zshrc   # 或 source ~/.bashrc / source ~/.config/fish/config.fish
```

验证安装：

```bash
sac --version
```

---

## 5 分钟上手示例

先写一个示例 `commands.toml`，体验嵌套 folder、命令分类、模糊搜索的完整流程。

### 第一步：创建示例数据文件

将以下内容写入 `~/.sac/commands.toml`（文件不存在时 `sac` 会自动创建空文件，你也可以直接替换内容）：

```toml
# ~/.sac/commands.toml — 示例数据

# ── Folders ──────────────────────────────────────────────────────────────────

[[folders]]
id = "devops"
parent = ""
name = "DevOps"

[[folders]]
id = "devops.k8s"
parent = "devops"
name = "Kubernetes"

[[folders]]
id = "devops.k8s.debug"
parent = "devops.k8s"
name = "Debug"

[[folders]]
id = "devops.docker"
parent = "devops"
name = "Docker"

[[folders]]
id = "git"
parent = ""
name = "Git"

[[folders]]
id = "git.branch"
parent = "git"
name = "Branch 操作"

# ── Commands ─────────────────────────────────────────────────────────────────

[[commands]]
id = 1
folder = "devops.k8s"
cmd = "kubectl get pods -n {namespace}"
desc = "列出指定 namespace 下的所有 pods"
comment = "需要提前配置好 kubectl context"
tags = ["k8s", "pods", "list"]
last_used = ""

[[commands]]
id = 2
folder = "devops.k8s"
cmd = "kubectl logs -f {pod} -n {namespace}"
desc = "实时跟踪 pod 日志"
comment = ""
tags = ["k8s", "logs"]
last_used = ""

[[commands]]
id = 3
folder = "devops.k8s.debug"
cmd = "kubectl exec -it {pod} -n {namespace} -- /bin/bash"
desc = "进入 pod 容器的 bash shell"
comment = "某些镜像只有 sh，把 bash 改成 sh"
tags = ["k8s", "debug", "exec"]
last_used = ""

[[commands]]
id = 4
folder = "devops.k8s.debug"
cmd = "kubectl describe pod {pod} -n {namespace}"
desc = "查看 pod 详情，排查 CrashLoopBackOff"
comment = ""
tags = ["k8s", "debug", "describe"]
last_used = ""

[[commands]]
id = 5
folder = "devops.docker"
cmd = "docker ps -a --format 'table {{.Names}}\t{{.Status}}\t{{.Ports}}'"
desc = "格式化展示所有容器状态"
comment = ""
tags = ["docker", "list"]
last_used = ""

[[commands]]
id = 6
folder = "devops.docker"
cmd = "docker logs -f --tail 100 {container}"
desc = "实时查看容器最后 100 行日志"
comment = ""
tags = ["docker", "logs"]
last_used = ""

[[commands]]
id = 7
folder = "git"
cmd = "git log --oneline --graph --decorate --all"
desc = "以图形方式展示所有分支历史"
comment = "适合用来理解分支结构"
tags = ["git", "log", "graph"]
last_used = ""

[[commands]]
id = 8
folder = "git.branch"
cmd = "git branch -vv"
desc = "查看所有本地分支及其 upstream 关联"
comment = ""
tags = ["git", "branch"]
last_used = ""

[[commands]]
id = 9
folder = "git.branch"
cmd = "git fetch --prune && git branch -r | grep -v '->' | while read remote; do git branch --track \"${remote#origin/}\" \"$remote\" 2>/dev/null; done"
desc = "同步所有远端分支到本地"
comment = "慎用，会拉取所有远端分支"
tags = ["git", "branch", "sync"]
last_used = ""
```

### 第二步：运行 `sac`，进入 TUI 浏览

```
sac
```

你会看到这样的界面（浏览模式，根目录）：

```
┌─ sac ──────────────────────────────────────────────────────────────┐
│  [数字] 直接选择  [↑↓] 移动  [Enter] 确认  [q/ESC] 返回/退出       │
├────────────────────────────────────────────────────────────────────┤
│  🔍                                                    浏览模式     │
├────────────────────────────────────────────────────────────────────┤
│                                                                    │
│  📁  [1]  DevOps                                                   │
│  📁  [2]  Git                                                      │
│  ──────────────────────────────────────────────────────            │
│  （根目录下暂无直接命令）                                            │
│                                                                    │
│  2 folders · 0 commands                                            │
└────────────────────────────────────────────────────────────────────┘
```

按 `1` 进入 DevOps，再按 `1` 进入 Kubernetes，你会看到：

```
┌─ sac — DevOps > Kubernetes ────────────────────────────────────────┐
│  [数字] 直接选择  [↑↓] 移动  [Enter] 确认  [q/ESC] 返回/退出       │
├────────────────────────────────────────────────────────────────────┤
│  🔍                                                                │
├────────────────────────────────────────────────────────────────────┤
│                                                                    │
│  📁  [1]  Debug                                                    │
│  ──────────────────────────────────────────────────────            │
│  $   [1]  kubectl get pods -n {namespace}                          │
│           列出指定 namespace 下的所有 pods                          │
│  $   [2]  kubectl logs -f {pod} -n {namespace}                     │
│           实时跟踪 pod 日志                                         │
│                                                                    │
│  1 folder · 2 commands                                             │
└────────────────────────────────────────────────────────────────────┘
```

### 第三步：模糊搜索体验

在任意界面直接打字，立即触发全局模糊搜索（无需按 `/` 或其他键）。

**示例：输入 `log`**

```
┌─ sac ──────────────────────────────────────────────────────────────┐
│  [数字] 直接选择  [↑↓] 移动  [Enter] 确认  [ESC] 清空搜索          │
├────────────────────────────────────────────────────────────────────┤
│  🔍 log                                               模糊搜索      │
├────────────────────────────────────────────────────────────────────┤
│                                                                    │
│  $  [1]  kubectl logs -f {pod} -n {namespace}                      │
│          📁 DevOps > Kubernetes                                    │
│          实时跟踪 pod 日志                                          │
│                                                                    │
│  $  [2]  docker logs -f --tail 100 {container}                     │
│          📁 DevOps > Docker                                        │
│          实时查看容器最后 100 行日志                                 │
│                                                                    │
│  $  [3]  git log --oneline --graph --decorate --all                │
│          📁 Git                                                    │
│          以图形方式展示所有分支历史                                  │
│                                                                    │
│  3 个结果                                                           │
└────────────────────────────────────────────────────────────────────┘
```

> 搜索 `log` 跨越了 Kubernetes、Docker、Git 三个不同 folder，结果按相关性排序：
> - `kubectl logs` 和 `docker logs` 的 `cmd` 字段直接包含 `log`，优先级最高
> - `git log` 的 `cmd` 字段也包含 `log`，同级排列
> - 再继续输入 `klog`，结果立即缩窄，`kubectl logs` 排到第一

**示例：输入 `dbg`（模糊匹配 "debug"）**

```
│  🔍 dbg                                               模糊搜索      │
│                                                                    │
│  $  [1]  kubectl exec -it {pod} -n {namespace} -- /bin/bash       │
│          📁 DevOps > Kubernetes > Debug                           │
│          进入 pod 容器的 bash shell                                 │
│                                                                    │
│  $  [2]  kubectl describe pod {pod} -n {namespace}                │
│          📁 DevOps > Kubernetes > Debug                           │
│          查看 pod 详情，排查 CrashLoopBackOff                       │
```

> `dbg` 没有出现在任何命令文本中，但 nucleo-matcher 的跳跃匹配识别出 `d-b-g` 是 `debug` 的字符跳跃子序列，精准命中 Debug folder 下的命令。

**示例：输入 `//kubectl exec`（精确搜索模式，以 `//` 开头）**

```
│  🔍 //kubectl exec                                    精确搜索      │
│                                                                    │
│  $  [1]  kubectl exec -it {pod} -n {namespace} -- /bin/bash       │
│          📁 DevOps > Kubernetes > Debug                           │
│          进入 pod 容器的 bash shell                                 │
```

> `//` 前缀切换为精确子字符串匹配，只返回命令内容中完整包含 `kubectl exec` 的结果。

### 第四步：选中命令，填入终端

在任意搜索结果页，按对应数字（如 `1`）或 Enter，该命令会直接出现在你的终端输入栏：

```bash
$ kubectl logs -f {pod} -n {namespace}█
```

你可以直接把 `{pod}` 和 `{namespace}` 替换成实际值，再按 Enter 执行。`sac` 本身不执行任何命令。

---

## 配置文件详解

`sac` 首次运行时自动创建 `~/.sac/config.toml`，默认内容如下：

```toml
# sac 主配置文件
# 路径：~/.sac/config.toml

[general]
# 每天首次启动时自动检查远端命令更新
# 仅在 commands_source.mode = "remote" 且 url 非空时生效
# 无网络时静默跳过，不影响正常启动
auto_check_remote = true

# 上次检查的日期，由 sac 自动维护，请勿手动修改
last_check = ""

[commands_source]
# 命令来源模式
# "local"  — 从本地文件读取（默认）
# "remote" — 从远端 URL 读取，适合团队共享或多设备同步
mode = "local"

# mode = "local" 时使用此路径
# 支持 ~ 展开
path = "~/.sac/commands.toml"

# mode = "remote" 时使用此 URL
# 支持 GitHub Gist raw URL、S3 public URL、任意公开 HTTP(S) 地址
# 示例：https://gist.githubusercontent.com/yourname/xxxx/raw/commands.toml
url = ""

[shell]
# 你使用的 shell 类型，影响 sac install 写入的集成脚本目标文件
# 可选值："zsh" / "bash" / "fish"
type = "zsh"
```

查看当前配置：

```bash
sac config
```

修改配置项（无需手动编辑文件）：

```bash
sac config set general.auto_check_remote false
sac config set commands_source.mode remote
sac config set commands_source.url https://gist.githubusercontent.com/yourname/xxx/raw/commands.toml
sac config set shell.type zsh
```

---

## commands.toml 格式

`~/.sac/commands.toml` 由两张扁平表组成，用 `parent` 字段表达层级关系，直接用文本编辑器维护也非常清晰。

### 完整字段说明

**Folder 字段**

| 字段 | 类型 | 必填 | 说明 |
|------|------|:----:|------|
| `id` | string | ✅ | 唯一标识符，用点分隔表达层级，如 `devops.k8s.debug` |
| `parent` | string | ✅ | 父 folder 的 `id`；根目录 folder 填 `""` |
| `name` | string | ✅ | 在 TUI 中显示的名称 |

**Command 字段**

| 字段 | 类型 | 必填 | 说明 |
|------|------|:----:|------|
| `id` | integer | ✅ | 唯一数字 ID，建议从 1 递增 |
| `folder` | string | ✅ | 所属 folder 的 `id` |
| `cmd` | string | ✅ | 命令内容，用 `{placeholder}` 标记需要替换的参数 |
| `desc` | string | ✅ | 命令描述，显示在 TUI 中，也参与搜索 |
| `comment` | string | ❌ | 额外备注，仅参与搜索，不在 TUI 主列表展示 |
| `tags` | string[] | ❌ | 标签列表，参与搜索权重计算 |
| `last_used` | string | ❌ | 最近使用时间（ISO 8601），由 sac 自动维护 |

### 层级约束

- 每个 folder 最多 **10 个直接子 folder**（键位 `1`–`9` 和 `0`）
- 每个 folder 最多 **10 条直接命令**（键位 `1`–`9` 和 `0`）
- folder 编号与 command 编号**各自独立**，互不影响

### 最小化模板

```toml
# ~/.sac/commands.toml — 最小化模板，复制后按需填充

[[folders]]
id = "my-folder"
parent = ""
name = "我的命令集"

[[commands]]
id = 1
folder = "my-folder"
cmd = "echo hello"
desc = "打个招呼"
comment = ""
tags = []
last_used = ""
```

---

## 搜索权重说明

模糊搜索的排序逻辑从高到低依次为：

| 优先级 | 规则 |
|--------|------|
| 1 | `cmd` 字段**精确包含**搜索词（大小写不敏感） |
| 2 | `desc` 字段精确包含搜索词 |
| 3 | 加权模糊评分：`cmd × 3`，`desc × 2`，`comment × 1`，`tags × 1` |
| 4 | 同分时：`last_used` 越近排越前 |
| 5 | 从未使用时：`id` 越小排越前 |

---

## 键位速查

### 浏览模式（搜索框为空）

| 按键 | 行为 |
|------|------|
| `1`–`9` / `0` | 直接跳转对应编号（folder 进入，command 选中退出） |
| `↑` / `↓` | 移动光标 |
| `Enter` | 确认当前高亮项 |
| `q` / `Esc` | 有父级则返回上一层；在根目录则退出 |
| 任意字符 | 追加到搜索框，立即进入模糊搜索模式 |

### 搜索模式（搜索框有内容）

| 按键 | 行为 |
|------|------|
| 继续输入 | 追加字符，实时更新结果 |
| `1`–`9` / `0` | 立即选中对应编号的结果，填入终端并退出 |
| `↑` / `↓` | 在结果列表中移动 |
| `Enter` | 确认当前高亮结果 |
| `Backspace` | 删除最后一个字符（正确处理中文多字节） |
| `Esc` | 清空搜索框，回到浏览模式 |

---

## CLI 命令速查

```bash
sac                              # 进入 TUI（唯一查找入口）

sac add                          # 交互式添加命令
sac add --folder <folder-id>     # 在指定 folder 下添加

sac new-folder <name>                       # 在根目录新建 folder
sac new-folder <name> --parent <folder-id>  # 在子目录新建

sac edit <command-id>            # 交互式编辑命令
sac delete <command-id>          # 删除命令（二次确认）

sac sync                         # 检查远端更新
sac sync --force                 # 强制用远端覆盖本地

sac config                       # 显示当前配置
sac config set <key> <value>     # 修改配置项

sac where config                 # 显示配置文件路径
sac where commands               # 显示命令文件路径

sac install                      # 写入 shell 集成脚本

sac export <path>                # 导出 commands.toml
sac import <path>                # 从文件导入命令
```

---

## Roadmap

- [ ] **`{placeholder}` 交互式填值**：在 TUI 中选中命令后，对 `{namespace}` 等占位符弹出填写提示，直接生成完整命令
- [ ] **命令使用频率统计**：记录每条命令的使用次数，在搜索排序中提升高频命令的权重
- [ ] **多命令文件支持**：允许同时挂载多个 commands.toml（个人 + 团队）
- [ ] **TUI 内联编辑**：在 TUI 中直接编辑命令，无需跳出到 CLI
- [ ] **颜色主题支持**：支持自定义 TUI 配色方案
- [ ] **命令版本历史**：记录命令修改历史，支持回滚
- [ ] **macOS / Linux 包管理器分发**：支持 `brew install sac`、AUR 等

---

## 贡献

欢迎 PR 和 Issue。提交前请确保：

```bash
cargo test        # 全部通过
cargo clippy      # 零警告
```

---

## License

MIT License © 2026 [handsomevictor](https://github.com/handsomevictor)

完整授权文本见 [LICENSE](LICENSE)。
