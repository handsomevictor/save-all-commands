# sac — save-all-commands

一个基于终端的命令管理工具，提供 TUI 界面，帮助你保存、搜索和复用常用命令。

---

## 特性

- **TUI 界面**：全键盘操作的终端用户界面，支持树状 folder 导航
- **模糊搜索**：基于 nucleo-matcher 的加权模糊搜索，快速定位命令
- **精确搜索**：使用 `//` 前缀触发精确字符串匹配
- **Shell 集成**：支持 zsh、bash、fish，选中命令后直接填入终端输入栏
- **远端同步**：通过 HTTP 与远端命令库同步，支持 diff 预览和用户确认
- **导入/导出**：以 TOML 格式导入或导出命令数据
- **层级结构**：命令按 folder 分组，支持嵌套子 folder

---

## 安装

确保已安装 Rust 工具链（推荐通过 [rustup](https://rustup.rs/) 安装），然后执行：

```bash
git clone https://github.com/handsomevictor/save-all-commands.git
cd save-all-commands
cargo install --path .
```

安装完成后，`sac` 命令即可在终端中直接使用。

---

## 快速上手

### 第一步：配置 Shell 集成

```bash
sac install
```

按提示选择你的 shell 类型（zsh / bash / fish），`sac` 会自动写入集成脚本。配置完成后，重启终端或执行 `source ~/.zshrc`（或对应配置文件）使其生效。

Shell 集成启用后，在 TUI 中选中命令并按 Enter，该命令会直接出现在你的终端输入栏，供你确认后执行。

### 第二步：进入 TUI

```bash
sac
```

启动后进入浏览模式，以树状结构展示所有 folder 和命令。

### 第三步：基本操作

**浏览模式**

| 键位 | 功能 |
|------|------|
| `↑` / `k` | 向上移动 |
| `↓` / `j` | 向下移动 |
| `Enter` | 选中命令，写入终端输入栏 |
| `/` | 进入搜索模式 |
| `q` | 退出 TUI |

**搜索模式**

| 键位 | 功能 |
|------|------|
| 输入字符 | 实时过滤命令列表 |
| `//` 前缀 | 切换为精确搜索 |
| `↑` / `↓` | 在结果中移动 |
| `Enter` | 选中命令，写入终端输入栏 |
| `Esc` | 退出搜索模式，返回浏览模式 |

---

## 配置文件

配置文件位于：

```
~/.sac/config.toml
```

查看配置文件路径：

```bash
sac where config
```

---

## 命令存储文件

所有保存的命令存储于：

```
~/.sac/commands.toml
```

查看命令文件路径：

```bash
sac where commands
```
