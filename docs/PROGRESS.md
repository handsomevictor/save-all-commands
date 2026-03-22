# 更新记录

## v0.1.0 — 2026-03-23

### 新增

- 数据层：Store（commands.toml 读写）、Config（config.toml 读写）
- 搜索层：模糊搜索（nucleo-matcher 加权评分）、精确搜索（// 前缀触发）
- TUI 层：浏览模式（树状 folder 导航）、搜索模式（实时过滤）
- CLI 子命令：add、new-folder、edit、delete、sync、config、where、install、export、import
- Shell Integration：zsh/bash/fish 三种 shell 支持，sac install 一键安装
- 同步层：远端 HTTP 同步、diff 展示、用户确认写入
- 完整测试套件：41 个测试用例，全部通过
