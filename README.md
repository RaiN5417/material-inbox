<p align="right">简体中文 | <a href="README.en.md">English</a></p>

# Download Inbox

> Downloads 文件夹不该是个杂物抽屉。

一个极轻量、本地优先的 Windows 小工具:在文件刚下载完成、你还记得它是干什么的时候,弹出一张不抢焦点的悬浮卡片,让你一秒决定它该去哪——不是靠 AI 猜。

- 不用 AI
- 不联网、不上云
- 不需要账号
- 不在后台轮询扫描目录(纯事件驱动监听)
- 每一次归档都可撤销;任何时候都不会自动永久删除文件

完整的产品/技术方案(本仓库的权威依据):
[docs/download_inbox_product_technical_spec_v0.2.md](docs/download_inbox_product_technical_spec_v0.2.md)。

## 下载

去 [Releases 页面](https://github.com/RaiN5417/material-inbox/releases/latest) 下载最新版——不需要装任何构建工具或额外依赖:

- **`Download-Inbox-x64-portable.zip`** —— 解压后直接运行 `download-inbox.exe`,免安装、不需要管理员权限。
- **`Download Inbox_x64-setup.exe`** —— 普通 Windows 安装程序(开始菜单快捷方式、可卸载)。已经把 WebView2 运行时安装包打进去了,就算这台机器上没装 WebView2 也能自动装好,不用你额外去找、去装。

首次运行 Windows 可能会弹 SmartScreen 警告(还没做代码签名),点"更多信息" → "仍要运行"就行。

## 当前状态

**MVP 功能已全部完成**(对应 spec 第 49 节里程碑计划的 M0–M6)。
打磨阶段(M7——README、文档、性能实测、首个 Release)正在进行;还剩什么没做见
[docs/architecture.md](docs/architecture.md)。

## 它能做什么

- 监听你的 Downloads 文件夹(用 `notify`,纯事件驱动,不轮询),会先判断文件是不是真的下载完了(靠 size/mtime 稳定性检测,`.crdownload`/`.part`/`.tmp` 这类临时文件一律忽略),确认完成才会有动作。
- 在鼠标所在的那块屏幕右下角弹出一张小小的、**不抢焦点**的悬浮卡片。点一下就能把文件归到某个 **分组**(目标文件夹)、标成 **临时文件**(到期自动进入待清理队列,绝不自动删除),或者选 **稍后处理**(留到主窗口再说)。
- 如果好几个文件几乎同时下载完,会合并成一张 **批量卡片**,不会一次弹一堆窗口。
- 每一次移动都有 **日志记录、可撤销**——同名文件绝不覆盖,自动加 `(1)`、`(2)` 后缀。
- **临时文件** 到期后进入待清理队列,你可以选择再留几天、归到某个分组,或者移进回收站(应用内部永远不会帮你彻底删除文件)。
- 常驻系统托盘,关闭主窗口只是隐藏,不会退出。

## 为什么做这个

文件下载下来的时候,根本没留下"为什么下载它"的记录。过几天再看,早忘了当初的上下文,而按扩展名自动分类这种做法,只是把混乱从一个文件夹搬到另一个文件夹。这个项目把"整理"这件事提前:趁你还记得的时候,问你一次,而不是等你忘了以后再整理。

## 架构

```text
React UI (apps/desktop/src)
        │  Tauri IPC + 事件
        ▼
Tauri / Rust 核心 (apps/desktop/src-tauri)
        ├── file-watcher      — 系统文件事件,纯事件驱动
        ├── download-detector — 判断这个路径是不是一个已经下载完、可操作的文件
        ├── event-engine      — 把短时间内一起下载完的文件合并通知,20 个文件不等于 20 张卡片
        ├── file-operations   — 唯一被允许移动/重命名/删除文件的模块
        ├── storage           — SQLite 连接池 + 迁移 + 仓储层
        └── domain            — 纯领域模型,不依赖 Tauri/SQLite/Windows API
```

详细文档:[docs/architecture.md](docs/architecture.md) · [docs/data-model.md](docs/data-model.md) ·
[docs/event-flow.md](docs/event-flow.md) · [docs/performance.md](docs/performance.md) ·
架构决策记录在 [docs/adr/](docs/adr/)。

## 从源码构建

前置依赖(都不是内置的,需要各自安装一次):

1. [Rust](https://rustup.rs/)(stable;需要 `rustfmt` + `clippy` 组件)
2. [Node.js 22.13+](https://nodejs.org/)(pnpm 11 要求这个版本)
3. `corepack enable`(Node 自带 pnpm)或 `npm i -g pnpm`
4. [Tauri 的 Windows 前置依赖](https://v2.tauri.app/start/prerequisites/) ——
   WebView2(大多数 Windows 10/11 已预装)和 MSVC C++ 构建工具

然后:

```bash
pnpm install --dir apps/desktop
pnpm --dir apps/desktop tauri dev
```

`cargo build` / `cargo test` 在仓库根目录直接对整个 workspace 生效,不需要碰前端。
`apps/desktop/src-tauri/icons/` 里的图标目前是占位图——正式发布前看一下那里的 README。

## 仓库结构

```text
apps/desktop/          Tauri + React 前端
crates/                domain、storage、file-watcher、download-detector、
                        event-engine、file-operations
migrations/            SQLite 表结构
docs/                   架构、数据模型、ADR、性能数据、完整 spec
.github/                CI、issue/PR 模板
```

## 不做什么

不做 AI 分类、不做 OCR、不做云同步、不做团队协作账号体系、不做全盘文件整理、不做自动删除。完整清单见 spec 第 4 节。

## 许可证

[MIT](LICENSE)
