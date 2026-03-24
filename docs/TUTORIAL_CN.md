# 完整功能使用说明

## 1. 安装与初始化

### 前置要求

通过 [rustup](https://rustup.rs/) 安装 Rust 工具链：

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 编译与安装

```bash
git clone https://github.com/handsomevictor/save-all-commands.git
cd save-all-commands
cargo install --path .
```

`sac` 安装到 `~/.cargo/bin/sac`，请确保 `~/.cargo/bin` 在你的 `PATH` 中。

### 首次运行

```bash
sac
```

首次运行时，`sac` 会自动创建 `~/.sac/` 目录，生成默认配置和空命令文件。如需快速体验，可加载内置示例数据：

```bash
mkdir -p ~/.sac
cp /path/to/save-all-commands/commands.toml.example ~/.sac/commands.toml
```

---

## 2. Shell 集成（sac install）

Shell 集成是 `sac` 最核心的功能。启用后，在 TUI 中选中命令，命令会直接填入当前终端的输入栏，而不是被执行。

### 安装

```bash
sac install
```

`sac` 会将集成脚本写入对应 shell 的 rc 文件，并显示写入路径。完成后重新加载：

```bash
source ~/.zshrc         # zsh
source ~/.bashrc        # bash
source ~/.config/fish/config.fish   # fish
```

### 工作原理

集成采用 tmpfile 方案，避免了 `$()` 子 shell 与 ZLE 的冲突：

```bash
# zsh 集成简化版
sac() {
  if [[ $# -eq 0 ]]; then
    local tmp
    tmp=$(mktemp)
    command sac >"$tmp" 2>/dev/tty
    local result
    result=$(<"$tmp")
    rm -f "$tmp"
    if [[ -n "$result" ]]; then
      if zle; then
        BUFFER=$result
        CURSOR=${#BUFFER}
        zle redisplay
      else
        print -z -- "$result"
      fi
    fi
  else
    command sac "$@"
  fi
}
```

核心特性：
- `sac` 运行在**前台**（无 `$()` 包裹），ZLE 不拦截 stdin
- stdout 重定向到 tmpfile，TUI 渲染走 `/dev/tty`
- 用 `if zle` 检测 ZLE 上下文，再设置 `BUFFER`
- 子命令（`sac add`、`sac --version` 等）直接 passthrough

### 升级集成

如已安装旧版集成，`sac install` 会自动检测旧格式 snippet 并进行替换。

---

## 3. TUI 使用

### 启动

```bash
sac
```

TUI 有两种模式：**浏览模式**和**搜索模式**。

### 浏览模式

默认模式。以编号列表显示当前 folder 的内容：子 folder 在前（青色，带 📁 图标），命令在后。

| 按键 | 功能 |
|------|------|
| `1`–`9` / `0` | 直接跳转到对应编号的项目 |
| `↑` / `↓` | 移动光标 |
| `Enter` | 进入 folder 或选中命令 |
| `q` / `Esc` | 返回上一层；在根目录则退出 |
| 任意可打印字符 | 追加到搜索框，进入搜索模式 |
| `Ctrl+C` | 退出且不输出任何命令 |

### 搜索模式

打字即自动进入搜索模式，实时跨 `cmd`、`desc`、`comment`、`tags` 搜索所有命令。

| 按键 | 功能 |
|------|------|
| 继续输入 | 追加字符，实时更新结果 |
| `1`–`9` / `0` | 立即选中对应编号的结果 |
| `↑` / `↓` | 在结果列表中移动光标 |
| `Enter` | 确认当前高亮结果 |
| `Backspace` | 删除最后一个字符（正确处理 Unicode） |
| `Esc` | 清空搜索框，回到浏览模式 |
| `Ctrl+C` | 退出且不输出任何命令 |

### vim 风格激活

输入 `/` 作为第一个字符会激活搜索模式（vim 风格）。`/` 会从实际查询中自动剥去，所以 `/doc` 等同于搜索 `doc`。要进入精确搜索模式，请输入 `//query`。

### 精确搜索（// 前缀）

以 `//` 开头的查询切换为精确子字符串匹配，无模糊评分，只返回拼合文本中包含完整字面字符串的命令。

```
//kubectl exec    →  只返回包含 "kubectl exec" 字面字符串的命令
```

### 选中命令

按数字键或在高亮结果上按 Enter，命令文本写入 shell 输入缓冲区。你可以：
- 将 `{placeholder}` 替换为实际参数
- 调整 flags
- 按 Enter 执行——或按 `Ctrl+C` 放弃

`sac` 本身不会执行任何命令。

---

## 4. 管理命令

### 添加命令

```bash
sac add                          # 交互式提示输入 folder、cmd、desc、comment、tags
sac add --folder <folder-id>     # 预选 folder
```

### 编辑命令

```bash
sac edit <command-id>
```

打开交互式提示，当前字段值预填充。按 Enter 保留原值。

### 删除命令

```bash
sac delete <command-id>
```

删除前需要确认。

---

## 5. 管理 Folder

### 创建 Folder

```bash
sac new-folder <name>                        # 在根目录创建
sac new-folder <name> --parent <folder-id>  # 在指定父 folder 下创建
```

### 层级约束

每个 folder 最多 **10 个直接子项**（子 folder + command 合计），与 TUI 的 `1`–`9` / `0` 键位一一对应。超过限制的操作会被拒绝并报错。

### 查找 folder ID

Folder ID 使用点分隔层级命名。根目录 folder 的 `id = ""`。通过 TUI 浏览或 `sac where commands` 查看文件路径，直接读取 ID。

---

## 6. 配置管理

### 查看当前配置

```bash
sac config
```

输出 `~/.sac/config.toml` 的完整内容。

### 修改配置

```bash
sac config set <key> <value>
```

支持的配置键：

| 键 | 说明 | 可选值 |
|----|------|--------|
| `general.auto_check_remote` | 每天首次启动时自动检查远端更新 | `true` / `false` |
| `commands_source.mode` | 命令来源 | `local` / `remote` |
| `commands_source.path` | 本地文件路径 | 任意路径（支持 `~`） |
| `commands_source.url` | 远端 TOML URL | 任意 HTTP/HTTPS URL |
| `shell.type` | shell 类型 | `zsh` / `bash` / `fish` |

### 示例

```bash
# 切换到远端模式并设置 GitHub Gist URL
sac config set commands_source.mode remote
sac config set commands_source.url https://gist.githubusercontent.com/yourname/xxx/raw/commands.toml

# 关闭启动时自动检查
sac config set general.auto_check_remote false

# 更改 shell 类型
sac config set shell.type bash
```

---

## 7. 查看文件路径

```bash
sac where config     # 打印 config.toml 的绝对路径
sac where commands   # 打印 commands.toml 的绝对路径
```

路径已展开（无 `~` 缩写），适用于脚本、备份或直接在编辑器中打开。

---

## 8. 导入/导出

### 导出

```bash
sac export ~/backup/commands.toml
```

将当前命令文件复制到指定路径。

### 导入

```bash
sac import ~/backup/commands.toml
```

校验文件格式后替换当前命令文件。格式不合法的文件在写入前即被拒绝。

---

## 9. 远端同步

### 手动同步

```bash
sac sync
```

1. 从 `commands_source.url` 获取 TOML 文件
2. 解析并校验
3. 与本地数据进行 diff
4. 展示新增、修改、删除的命令
5. 提示用户确认
6. 确认后写入

### 强制同步

```bash
sac sync --force
```

跳过确认提示，直接用远端数据覆盖本地。**注意**：本地未同步的修改将会丢失。

### 启动时自动检查

当 `general.auto_check_remote = true`（默认值）且 `commands_source.mode = "remote"` 时，`sac` 每天首次启动时自动检查远端更新。无网络时静默跳过，不影响正常使用。

---

## 10. commands.toml 格式参考

### Folder 字段

| 字段 | 类型 | 必填 | 说明 |
|------|------|:----:|------|
| `id` | string | ✅ | 唯一标识符，用点分隔表达层级，如 `devops.k8s.debug` |
| `parent` | string | ✅ | 父 folder 的 `id`；根目录 folder 填 `""` |
| `name` | string | ✅ | 在 TUI 中显示的名称 |

### Command 字段

| 字段 | 类型 | 必填 | 说明 |
|------|------|:----:|------|
| `id` | integer | ✅ | 唯一数字 ID，建议从 1 递增 |
| `folder` | string | ✅ | 所属 folder 的 `id` |
| `cmd` | string | ✅ | 命令内容，用 `{placeholder}` 标记需要替换的参数 |
| `desc` | string | ✅ | 命令描述，显示在 TUI 中，也参与搜索 |
| `comment` | string | | 额外备注，仅参与搜索，不在 TUI 主列表展示 |
| `tags` | string[] | | 标签列表，tag 匹配享有最高搜索优先级 |
| `last_used` | string | | ISO 8601 时间戳，由 sac 自动维护 |

### 多行命令

使用 TOML 字面字符串（三重单引号）存储跨行命令，反斜杠原样保留：

```toml
cmd = '''
aws s3 sync s3://{bucket}/{prefix}/ . \
  --exclude "*.tmp" \
  --delete
'''
```

### 最小化模板

```toml
[[folders]]
id     = "my-folder"
parent = ""
name   = "我的命令集"

[[commands]]
id        = 1
folder    = "my-folder"
cmd       = "echo hello"
desc      = "打个招呼"
comment   = ""
tags      = []
last_used = ""
```
