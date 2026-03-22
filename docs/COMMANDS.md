# 命令列表

## 进入 TUI

```bash
sac
```

---

## 添加命令

```bash
sac add
sac add --folder <folder-id>
```

---

## 管理 Folder

```bash
sac new-folder <name>
sac new-folder <name> --parent <folder-id>
```

---

## 编辑/删除命令

```bash
sac edit <command-id>
sac delete <command-id>
```

---

## 同步

```bash
sac sync
sac sync --force
```

---

## 配置

```bash
sac config
sac config set <key> <value>
sac config set general.auto_check_remote false
sac config set commands_source.mode remote
sac config set commands_source.url <url>
sac config set shell.type zsh
```

---

## 查看文件路径

```bash
sac where config
sac where commands
```

---

## Shell 集成

```bash
sac install
```

---

## 导入/导出

```bash
sac export <path>
sac import <path>
```
