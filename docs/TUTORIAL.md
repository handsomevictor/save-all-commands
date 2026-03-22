# 完整功能使用说明

## 1. 安装与初始化

### 安装

确保已安装 Rust 工具链（推荐通过 [rustup](https://rustup.rs/) 安装），然后执行：

```bash
git clone https://github.com/handsomevictor/save-all-commands.git
cd save-all-commands
cargo install --path .
```

安装成功后，`sac` 命令即可在任意目录使用。

### 首次运行

直接执行 `sac` 会启动 TUI 界面。若 `~/.sac/` 目录不存在，`sac` 会在首次运行时自动创建，并生成默认的配置文件和命令文件。

---

## 2. Shell 集成（sac install）

Shell 集成是 `sac` 的核心功能之一。启用后，在 TUI 中选中命令并按 Enter，该命令会直接填入当前终端的输入栏，让你确认后再执行。

### 安装 Shell 集成

```bash
sac install
```

执行后，`sac` 会提示你选择 shell 类型：

- `zsh`：向 `~/.zshrc` 写入集成脚本
- `bash`：向 `~/.bashrc` 写入集成脚本
- `fish`：向 Fish 配置目录写入集成脚本

选择完成后，重新加载 shell 配置：

```bash
# zsh
source ~/.zshrc

# bash
source ~/.bashrc

# fish
source ~/.config/fish/config.fish
```

### 验证集成

重新加载后，执行 `sac`，在 TUI 中选中任意命令按 Enter，若命令出现在终端输入栏，则集成成功。

---

## 3. TUI 使用

### 启动

```bash
sac
```

TUI 分为两种模式：**浏览模式**和**搜索模式**。

### 浏览模式

启动时默认进入浏览模式，以树状结构展示所有 folder 和命令。

| 键位 | 功能 |
|------|------|
| `↑` / `k` | 向上移动光标 |
| `↓` / `j` | 向下移动光标 |
| `Enter` | 选中当前命令，写入终端输入栏后退出 TUI |
| `/` | 进入搜索模式 |
| `q` / `Esc` | 退出 TUI |

Folder 节点可以展开/折叠，方便浏览大量命令。

### 搜索模式

按 `/` 后进入搜索模式，顶部会显示搜索输入框。

| 键位 | 功能 |
|------|------|
| 输入字符 | 实时过滤并排序命令列表 |
| `↑` / `↓` | 在搜索结果中移动光标 |
| `Enter` | 选中当前命令，写入终端输入栏后退出 TUI |
| `Backspace` | 删除最后一个搜索字符 |
| `Esc` | 清空搜索词，返回浏览模式 |

### 精确搜索（// 前缀）

在搜索模式下，以 `//` 开头输入搜索词，会切换为精确子字符串匹配，结果不经过模糊评分。

示例：

```
// git commit
```

仅显示命令内容中包含字面字符串 `git commit` 的结果。

---

## 4. 管理命令

### 添加命令（sac add）

交互式添加一条命令到根 folder：

```bash
sac add
```

添加命令到指定 folder：

```bash
sac add --folder <folder-id>
```

执行后，`sac` 会提示输入：
- 命令内容（必填）
- 命令名称/描述（可选）

### 编辑命令（sac edit）

```bash
sac edit <command-id>
```

通过命令 ID 编辑已存在的命令内容和描述。

### 删除命令（sac delete）

```bash
sac delete <command-id>
```

通过命令 ID 删除一条命令。删除前会要求确认。

---

## 5. 管理 Folder

### 创建 Folder（sac new-folder）

在根目录创建新 folder：

```bash
sac new-folder <name>
```

在指定父 folder 下创建子 folder：

```bash
sac new-folder <name> --parent <folder-id>
```

### 层级约束

每个 folder 最多包含：
- **10 个子 folder**
- **10 条命令**

超出限制时，`sac` 会拒绝操作并给出错误提示。

---

## 6. 配置管理

### 查看当前配置（sac config）

```bash
sac config
```

输出当前 `~/.sac/config.toml` 的所有配置项。

### 修改配置（sac config set）

```bash
sac config set <key> <value>
```

支持的配置键：

| 键 | 说明 | 示例值 |
|----|------|--------|
| `general.auto_check_remote` | 启动时自动检查远端更新 | `true` / `false` |
| `commands_source.mode` | 命令来源模式 | `local` / `remote` |
| `commands_source.url` | 远端命令源 URL | `https://example.com/commands.toml` |
| `shell.type` | Shell 类型 | `zsh` / `bash` / `fish` |

示例：

```bash
# 关闭自动检查远端
sac config set general.auto_check_remote false

# 切换为远端模式
sac config set commands_source.mode remote

# 设置远端 URL
sac config set commands_source.url https://example.com/commands.toml

# 设置 shell 类型
sac config set shell.type zsh
```

---

## 7. 文件路径查询（sac where）

查看配置文件的绝对路径：

```bash
sac where config
```

查看命令文件的绝对路径：

```bash
sac where commands
```

这对于手动编辑文件或备份数据非常有用。

---

## 8. 导入/导出

### 导出命令（sac export）

将当前命令数据导出为 TOML 文件：

```bash
sac export <path>
```

示例：

```bash
sac export ~/backup/commands_backup.toml
```

### 导入命令（sac import）

从 TOML 文件导入命令数据：

```bash
sac import <path>
```

示例：

```bash
sac import ~/backup/commands_backup.toml
```

导入时，`sac` 会校验文件格式，确保数据合法后再写入。

---

## 9. 远端同步

### 手动同步（sac sync）

从配置的远端 URL 拉取最新命令数据，与本地进行 diff 比对，展示差异后由用户确认是否写入：

```bash
sac sync
```

### 强制同步（sac sync --force）

跳过用户确认，直接用远端数据覆盖本地：

```bash
sac sync --force
```

**注意**：`--force` 会覆盖本地所有修改，使用前请确认已备份重要数据。

### 同步流程

1. `sac` 向 `commands_source.url` 发起 HTTP GET 请求，下载远端 TOML 数据
2. 解析远端数据，与本地数据进行 diff
3. 在终端展示新增、修改、删除的命令列表
4. 提示用户确认（`--force` 时跳过此步）
5. 用户确认后，将远端数据写入 `~/.sac/commands.toml`

---

## 10. 自动检查远端（auto_check_remote）

配置项 `general.auto_check_remote` 控制 `sac` 启动时是否自动检查远端更新。

- 默认值：`true`
- 设为 `false` 后，每次启动 TUI 时不再自动拉取远端数据
- 仍可通过 `sac sync` 手动触发同步

关闭自动检查：

```bash
sac config set general.auto_check_remote false
```

---

## 11. 数据格式说明（commands.toml 结构）

`~/.sac/commands.toml` 使用 TOML 格式存储命令数据，基本结构如下：

```toml
[[folders]]
id = "folder-uuid-1"
name = "Git"
parent_id = ""        # 空字符串表示根 folder

[[folders]]
id = "folder-uuid-2"
name = "Docker"
parent_id = ""

[[commands]]
id = "cmd-uuid-1"
name = "提交所有更改"
command = "git add . && git commit -m ''"
folder_id = "folder-uuid-1"

[[commands]]
id = "cmd-uuid-2"
name = "查看运行中的容器"
command = "docker ps"
folder_id = "folder-uuid-2"
```

字段说明：

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | string | UUID，唯一标识符 |
| `name` | string | 显示名称 / 描述 |
| `command` | string | 实际命令内容 |
| `folder_id` | string | 所属 folder 的 ID |
| `parent_id` | string | 父 folder 的 ID，根 folder 为空字符串 |

---

## 12. 层级约束说明

为保持界面可读性和性能，`sac` 对数据结构施加以下约束：

- 每个 folder 最多包含 **10 个直接子 folder**
- 每个 folder 最多包含 **10 条命令**

这些约束在以下操作时会被校验：
- `sac add`：添加命令时检查目标 folder 的命令数量
- `sac new-folder`：创建子 folder 时检查父 folder 的子 folder 数量
- `sac import` / `sac sync`：导入或同步数据时对整个数据集进行校验

超出限制时，操作会被拒绝并显示错误信息。
