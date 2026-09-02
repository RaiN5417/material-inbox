# Download Inbox — 产品方案与技术方案
> 面向 Agent / AI Coding Agent 的单文件规格说明  
> 文档版本：v0.1  
> 日期：2026-08-28  
> 平台优先级：Windows 11 > Windows 10 > macOS/Linux（后续）  
> 项目性质：开源练手项目 / GitHub Star 项目 / 作品集 / 面试项目  
> 产品原则：No AI / Local-first / Lightweight / Human-in-the-loop

---

# 0. Agent 执行说明（必须先读）

本文件同时是：

1. 产品需求文档（PRD）
2. 技术设计文档（TDD）
3. Agent 开发约束
4. MVP 验收标准
5. Roadmap
6. 面试作品说明

## 0.1 权威顺序

当实现过程中出现冲突时，按以下优先级执行：

1. 本文档中的 **MUST / MUST NOT**
2. 本文档中的模块边界和状态机
3. 本文档中的验收标准
4. 本文档中的推荐技术选型
5. Agent 自行推断

Agent 不得为了“更先进”擅自引入 AI、云服务、微服务、Redis、PostgreSQL、Python 后端、Electron 或其他未要求的重型依赖。

## 0.2 核心实现原则

MUST：

- Windows-first。
- Local-first。
- 默认不联网。
- 默认不上传文件、文件名、路径或来源数据。
- 后台常驻核心必须使用 Rust。
- UI 使用 Tauri 2 + React + TypeScript。
- 本地数据库使用 SQLite。
- 文件系统监听必须使用事件驱动，不允许高频轮询 Downloads。
- 悬浮卡片不得抢占当前应用输入焦点。
- 文件操作必须可撤销。
- 用户忽略弹窗时不得阻塞下载或文件访问。
- 必须正确处理浏览器临时下载文件，如 `.crdownload` / `.part`。
- MVP 不允许删除用户文件；Temporary 到期仅进入“待清理”或系统回收站流程。
- 所有危险文件操作必须经过统一 `file-operations` 模块。

MUST NOT：

- 不使用 AI 自动分类。
- 不使用 LLM。
- 不使用向量数据库。
- 不要求登录。
- 不要求云账户。
- 不把“图片 → Images、PDF → Documents”作为核心产品价值。
- 不通过后台循环扫描目录来替代文件事件监听。
- 不直接永久删除文件。
- 不在 UI 层直接操作文件系统。
- 不让 React 保存业务真相；业务状态以 Rust Core + SQLite 为准。
- 不把所有逻辑塞进 `src-tauri/src/main.rs`。

---

# 1. 产品一句话定义

> **一个极轻量、本地优先的下载文件 Inbox：在文件刚下载完成、用户还记得它是干什么的时候，通过不抢焦点的桌面悬浮卡片，让用户一键决定它属于哪个项目、是否只是临时文件，或稍后处理。**

英文定位：

> **A tiny, local-first inbox for downloads. Tell files where they belong while you still remember.**

核心 slogan 候选：

> **Don’t organize files later. Tell them where they belong while you still remember.**

---

# 2. 项目目标

## 2.1 产品目标

解决以下真实问题：

```text
Downloads/
├── image.png
├── image (1).png
├── 123.pdf
├── final.zip
├── setup.exe
├── 微信图片_20260828.jpg
├── 未命名 (3).png
└── ...
```

传统整理方式的问题：

- 文件产生时没有记录“为什么下载”。
- 过几天后用户已经忘记上下文。
- 自动按扩展名分类只是把混乱从一个目录移动到多个目录。
- 规则系统配置成本高。
- AI 分类可能猜错，而且不是这个产品需要解决的问题。
- 下载文件夹长期膨胀。
- 很多文件其实只是“一次性临时使用”。

本产品将整理时机前移：

```text
文件下载完成
      ↓
用户仍然记得上下文
      ↓
非抢焦点 Floating Card
      ↓
[项目 A] [项目 B] [临时] [稍后]
      ↓
1 秒完成上下文登记
```

---

# 3. 项目成功标准

这是一个开源作品，而不是商业 SaaS。

成功标准按优先级排序：

## P0 — 可演示

15 秒无解说 Demo GIF 能让陌生用户理解用途：

```text
Chrome 下载图片
→
右下角悬浮卡片出现
→
点击“Project A”
→
文件被归档
→
Undo 可恢复
```

## P1 — 工程质量

项目至少包含以下可以在面试中展开讨论的工程问题：

- OS 文件事件监听
- 文件下载完成判断
- Debounce
- 批量事件聚合
- 非抢焦点浮窗
- Rust 异步任务
- SQLite
- Transaction / Operation Log
- Undo
- Windows 系统托盘
- 开机启动
- 浏览器扩展与 Native Messaging（v0.2）
- 性能与内存优化
- CI / 测试

## P2 — GitHub 表现

目标不是保证 Star 数，而是提高项目可传播性：

- README 第一屏可理解
- 有 Demo GIF
- 有架构图
- 有 benchmark
- 有 Releases
- 有 Roadmap
- 有 Issue templates
- 有 `CONTRIBUTING.md`
- 有良好的 first issue

---

# 4. 非目标（Non-goals）

MVP 不做：

- AI 分类
- 自动识别项目
- OCR
- 文档语义理解
- 云同步
- 团队协作
- SaaS 账户系统
- NAS 文件管理
- 全盘文件整理
- Explorer 替代品
- Everything 替代品
- 文件内容搜索
- 虚拟文件系统
- Windows Shell Extension
- 自动永久删除
- macOS/Linux 正式支持
- 复杂 Hazel 式规则引擎
- 基于扩展名的“自动文件夹分类器”作为主功能

---

# 5. 目标用户

第一阶段目标用户：

- Windows 高频下载文件用户
- 设计师
- 开发者
- 内容创作者
- 项目型自由职业者
- 经常从浏览器/IM/协作工具下载临时资料的人
- Downloads 文件夹长期积累数百/数千文件的人

不是首要目标：

- 企业 DMS 用户
- 大型 NAS 用户
- 需要合规文件管理的组织
- 只偶尔下载文件的人

---

# 6. 产品核心原则

## 6.1 Human-in-the-loop

不猜用户意图。

用户下载文件时最清楚它属于什么，所以只需要在正确时机问一个非常轻的问题。

## 6.2 Context at creation time

核心不是“以后整理”，而是：

> 在文件产生的那一刻捕获上下文。

## 6.3 Zero-friction

弹窗必须满足：

- 不抢焦点
- 不阻塞
- 可忽略
- 可批量
- 1 秒内可完成主要操作

## 6.4 Local-first

默认：

```text
Network = 0
Account = 0
Cloud = 0
Telemetry = 0（如未来加入必须 opt-in）
```

## 6.5 Safe-by-default

文件管理程序的第一优先级不是聪明，而是不能丢文件。

---

# 7. MVP 功能范围

MVP / v0.1 只实现 7 个核心能力：

1. 监听 Downloads
2. 判断文件真正下载完成
3. 弹出非抢焦点 Floating Card
4. Group / Temporary / Later
5. 批量下载合并
6. SQLite 历史记录
7. Undo

辅助能力：

- 系统托盘
- 开机启动开关
- 主窗口 Inbox
- 设置 Downloads 路径
- 打开文件所在位置
- 打开文件
- 最近操作记录

---

# 8. 核心用户流程

## 8.1 单文件下载

```text
[浏览器]
用户点击下载
    ↓
Downloads 中出现临时文件
    ↓
Rust Watcher 收到 Create/Modify/Rename
    ↓
判断文件尚未完成
    ↓
等待稳定
    ↓
确认最终文件
    ↓
建立 InboxItem
    ↓
Floating Card 出现
    ↓
用户选择：
    ├── Group
    ├── Temporary
    └── Later
```

### Group

```text
选择 Project A
→ 执行归档策略
→ 写 Operation Log
→ 标记 Organized
→ Toast/Floating Card 消失
```

### Temporary

```text
选择 Temporary
→ 文件留在原位置或移动至 Temporary root（由设置决定）
→ 记录 expires_at
→ 到期后进入 Cleanup Queue
→ 不自动永久删除
```

### Later

```text
选择 Later
→ 不移动文件
→ InboxItem.status = pending
→ 悬浮卡片关闭
→ 主窗口 Inbox 中继续显示
```

---

# 9. 批量下载流程

必须避免 20 个文件弹 20 次。

## 9.1 Batch Window

默认聚合窗口：

```text
2.0 秒
```

可配置范围：

```text
500ms ~ 5000ms
```

聚合条件：

- 文件在同一 monitored root
- 创建时间接近
- 可选：来源相同（v0.2）
- 不包含仍未完成下载的文件

例：

```text
00:00.000 a.jpg
00:00.120 b.jpg
00:00.410 c.jpg
00:01.100 d.jpg

→ 一个 batch
```

UI：

```text
┌──────────────────────────────┐
│ ↓ 4 new files                │
│                              │
│ a.jpg                        │
│ b.jpg                        │
│ c.jpg                        │
│ d.jpg                        │
│                              │
│ [Project A] [Temporary]      │
│ [Review individually]        │
│ [Later]                      │
└──────────────────────────────┘
```

---

# 10. Floating Card UX 规格

Floating Card 是项目最重要的 UX。

## 10.1 行为

MUST：

- Frameless
- Always-on-top
- 默认不抢焦点
- 出现在当前主显示器工作区右下角
- 多显示器时优先跟随鼠标所在显示器
- 避免遮挡 Windows Taskbar
- DPI aware
- 支持动画
- 用户正在全屏游戏/演示时可暂停提示
- 点击卡片本身时才允许获取焦点
- Esc 可关闭
- 不操作时自动收起，但进入 Inbox

推荐默认：

```text
显示时长：8 秒
动画：150~220ms
宽度：360~420px
```

## 10.2 单文件卡片

```text
┌────────────────────────────────┐
│ ↓ New download                 │
│                                │
│ design-reference.png           │
│ PNG · 3.8 MB                   │
│                                │
│ Where does this belong?        │
│                                │
│ [ Project A ] [ Project B ]    │
│ [ Temporary ] [ Later ]        │
│                                │
│                        •••  ×   │
└────────────────────────────────┘
```

## 10.3 交互原则

禁止：

- 强制 Modal
- 阻止用户继续工作
- 默认弹到屏幕中央
- 每个文件都播放声音
- 下载高峰时刷屏
- 未经确认执行危险删除

---

# 11. 主窗口信息架构

主窗口不是日常必须打开的。

```text
Sidebar
├── Inbox
├── Temporary
├── Groups
├── History
└── Settings
```

## Inbox

显示：

- Pending
- Later
- Failed operations
- 未解决冲突

## Temporary

显示：

- 过期时间
- 文件大小
- 来源
- 延长保留
- 移入 Group
- 移到系统回收站

## Groups

MVP 一个文件只能有一个 Primary Group。

未来可增加 Tags，多对多。

## History

显示：

- Move
- Rename
- Mark temporary
- Restore
- Undo
- Error

---

# 12. 分组模型

MVP 分组 = 用户理解上的“项目/上下文”。

示例：

```text
邻里中心
茶叶包装
Side Project
Invoices
Temporary
```

Group 可配置：

```text
name
destination_path
icon
sort_order
is_pinned
```

MVP 不做复杂标签体系。

---

# 13. 文件生命周期状态机

每个文件必须使用明确状态，不允许靠 UI 推断。

本状态机与第 25 节「Temporary 生命周期」为同一状态机，此处为完整版（补全 PendingRetry / CleanupReady 后续分支 / Trashed 终态）。

```text
DETECTED
   ↓
WAITING_STABLE ──(size/mtime 变化)──→ 重置稳定计时器，回到 WAITING_STABLE
   │
   ├──(超过 max_wait，如 120s)──→ PENDING_RETRY ──(后台低频重试)──→ WAITING_STABLE
   │
   ↓ (连续 N 次稳定 + 可共享读取)
READY
   ↓
PENDING
   │
   ├──────────────→ ORGANIZING ──→ ORGANIZED
   │
   ├──────────────→ TEMPORARY
   │                   │
   │                   │ (到期，或用户手动标记到期)
   │                   ↓
   │                EXPIRED
   │                   ↓
   │              CLEANUP_READY ── 用户在 Temporary 面板选择：
   │                   │
   │        ┌──────────┼──────────────┬────────────────┐
   │        ↓          ↓              ↓                ↓
   │   Keep N more   Move to      Move to           Ignore
   │      days        Group      Recycle Bin      （仅关闭提醒，
   │        ↓          ↓              ↓             不改变状态，
   │   TEMPORARY   ORGANIZING     TRASHED           留在 CLEANUP_READY）
   │  (重置expires_at)  ↓        （终态，MVP 不在
   │               ORGANIZED      App 内提供 Undo，
   │                              可通过系统回收站
   │                              手动恢复）
   │
   └──────────────→ LATER ──(用户在主窗口 Inbox 后续处理)──→ ORGANIZING / TEMPORARY

任意可操作状态（ORGANIZED / TRASHED / ERROR 自身除外）
   ↓ 操作失败（preflight/execute/verify 任一步失败）
ERROR
   ↓ 用户重试或修复后
回到操作发生前的状态（如 PENDING / TEMPORARY / CLEANUP_READY）

ORGANIZED
   ↓ Undo
RESTORING
   ↓
PENDING / READY

任意状态（文件在磁盘上被 App 之外的操作移动/删除）
   ↓
MISSING
```

要点说明：

- **PENDING_RETRY** 与 `Error` 是两回事：前者是 stability check 超时后的可恢复重试状态（对应第 20.3 节），不需要用户介入；只有重试仍持续失败（如权限问题）才转入 `Error`。
- 原图中的 `IGNORED/LATER` 已拆开消歧义：
  - `LATER` 专指 Inbox 阶段用户选择「稍后处理」，文件保持原位，状态机语义与 `Pending` 平级。
  - Cleanup 阶段的「Ignore」不是一个独立状态，只是关闭当次提醒，文件继续停留在 `CLEANUP_READY`，下次进入 Temporary 面板仍会看到它。
- **CLEANUP_READY 不再是终点**：四个用户动作分别显式映射到 `TEMPORARY`（续期）、`ORGANIZING→ORGANIZED`（归档）、`TRASHED`（移入回收站）、原地停留（忽略）。
- 新增终态 **TRASHED**：文件已被移入系统回收站，App 不再追踪其物理位置，但 `files` 行和 `operations` 记录保留用于审计；MVP 范围内不提供「从回收站一键恢复」的 Undo（用户可自行从 Windows 回收站还原），这一点应写进 Undo 文档避免用户误以为 App 内 Undo 也能撤销这一步。

推荐枚举：

```rust
enum FileStatus {
    Detected,
    WaitingStable,
    PendingRetry,
    Ready,
    Pending,
    Organizing,
    Organized,
    Temporary,
    Expired,
    CleanupReady,
    Later,
    Trashed,
    Restoring,
    Error,
    Missing,
}
```

---

# 14. 技术架构总览

```text
┌─────────────────────────────────────────────┐
│                 React UI                    │
│      Main Window / Floating Card            │
└─────────────────────┬───────────────────────┘
                      │ Tauri IPC
                      ▼
┌─────────────────────────────────────────────┐
│               Tauri / Rust Core             │
│                                             │
│  ┌──────────────┐  ┌─────────────────────┐  │
│  │ File Watcher │→ │ Event / Batch Engine│  │
│  └──────────────┘  └──────────┬──────────┘  │
│                               │             │
│  ┌──────────────┐  ┌──────────▼──────────┐  │
│  │ File Ops     │← │ Inbox Service       │  │
│  └──────┬───────┘  └──────────┬──────────┘  │
│         │                      │             │
│         ▼                      ▼             │
│  ┌──────────────┐       ┌───────────────┐    │
│  │ Operation Log│       │ SQLite Store  │    │
│  └──────────────┘       └───────────────┘    │
└─────────────────────────────────────────────┘

v0.2 optional:

Chrome / Edge Extension
        │
        │ Native Messaging / stdio
        ▼
Native Messaging Host
        │
        ▼
Rust Core Source Context
```

---

# 15. 推荐技术栈

## Desktop UI

- Tauri 2
- React
- TypeScript
- Vite
- Tailwind CSS（可选；如引入则统一使用）

## Rust Core

- Rust stable
- Tokio
- notify
- serde / serde_json
- tracing
- thiserror / anyhow
- uuid
- chrono 或 time
- directories
- trash（系统回收站，可评估）

## Database

优先：

- SQLite
- SQLx

原因：

- 本地优先
- 单用户
- 无独立 DB 服务
- 数据量低
- SQL 可直接展示工程能力
- SQLx 支持 SQLite，且适合 Tokio async 架构

备选：

- rusqlite

不要同时引入 SQLx + rusqlite，避免重复依赖和 sqlite binding 冲突。

## File watcher

MVP：

- `notify` stable

不要默认采用 pre-release / RC。

如 Windows 特定边界出现严重问题，再评估：

- `ReadDirectoryChangesW`
- `windows` / `windows-sys`

但必须封装在 `file-watcher` crate 后面，不能让 Windows API 泄漏到业务层。

---

# 16. 当前技术基线（2026-08）

以下用于 Agent 选型，不要求长期写死精确版本：

- Tauri：2.x
- notify：8.2.x stable
- SQLx：0.9.x
- Chrome Extension：Manifest V3
- Native Messaging：Chrome 官方 stdio host 协议

依赖版本在项目初始化时使用稳定版本，并提交 lockfile。

---

# 17. Repository 结构

推荐 monorepo：

```text
download-inbox/
├── apps/
│   ├── desktop/
│   │   ├── src/
│   │   │   ├── components/
│   │   │   ├── features/
│   │   │   │   ├── inbox/
│   │   │   │   ├── floating-card/
│   │   │   │   ├── groups/
│   │   │   │   ├── temporary/
│   │   │   │   ├── history/
│   │   │   │   └── settings/
│   │   │   ├── lib/
│   │   │   └── main.tsx
│   │   └── src-tauri/
│   │       ├── src/
│   │       │   ├── app.rs
│   │       │   ├── commands/
│   │       │   └── lib.rs
│   │       ├── capabilities/
│   │       └── tauri.conf.json
│   │
│   └── browser-extension/          # v0.2
│       ├── src/
│       ├── manifest.json
│       └── package.json
│
├── crates/
│   ├── domain/
│   ├── file-watcher/
│   ├── download-detector/
│   ├── event-engine/
│   ├── file-operations/
│   ├── storage/
│   ├── inbox-service/
│   ├── source-context/             # v0.2
│   └── native-messaging-host/      # v0.2
│
├── migrations/
├── docs/
│   ├── architecture.md
│   ├── event-flow.md
│   ├── data-model.md
│   ├── performance.md
│   ├── security.md
│   └── adr/
│
├── benches/
├── tests/
├── .github/
│   ├── workflows/
│   ├── ISSUE_TEMPLATE/
│   └── PULL_REQUEST_TEMPLATE.md
│
├── README.md
├── CONTRIBUTING.md
├── SECURITY.md
├── LICENSE
└── Cargo.toml
```

MVP 若觉得 workspace 过重，可先只拆 4 个 crate：

```text
domain
file-watcher
file-operations
storage
```

但禁止所有逻辑集中在一个 crate 单文件内。

---

# 18. Rust 模块职责

## domain

纯领域模型，无 Tauri、SQLite、Windows API 依赖。

包含：

- FileRecord
- InboxItem
- Group
- Operation
- Batch
- enums
- domain errors

## file-watcher

负责：

- 启停监控
- normalize notify events
- path filtering
- 去除临时噪声
- 生成标准 `FsEvent`

输出示例：

```rust
struct FsEvent {
    path: PathBuf,
    kind: FsEventKind,
    observed_at: DateTime<Utc>,
}
```

## download-detector

负责判断：

> 这个路径是否已经成为一个可操作的最终文件？

不要把此逻辑写进 watcher。

## event-engine

负责：

- debounce
- dedupe events
- batch aggregation
- queue
- backpressure

## file-operations

唯一允许：

- move
- rename
- restore
- trash

所有操作：

1. preflight
2. create operation log
3. execute
4. verify
5. commit status

## storage

负责：

- migrations
- repository
- transaction
- SQLite pool
- query

## inbox-service

业务 orchestration：

```text
watch event
→ completion check
→ create InboxItem
→ batch
→ notify UI
→ accept user action
→ file operation
→ persist state
```

---

# 19. 数据模型

## 19.1 files

```sql
CREATE TABLE files (
    id TEXT PRIMARY KEY,
    original_name TEXT NOT NULL,
    current_name TEXT NOT NULL,
    original_path TEXT NOT NULL,
    current_path TEXT NOT NULL,
    extension TEXT,
    mime_type TEXT,
    size_bytes INTEGER,
    status TEXT NOT NULL,
    detected_at TEXT NOT NULL,
    ready_at TEXT,
    organized_at TEXT,
    last_seen_at TEXT NOT NULL,
    expires_at TEXT,
    group_id TEXT,
    source_context_id TEXT,
    error_code TEXT,
    error_message TEXT,
    FOREIGN KEY(group_id) REFERENCES groups(id)
);
```

## 19.2 groups

```sql
CREATE TABLE groups (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    destination_path TEXT,
    icon TEXT,
    is_pinned INTEGER NOT NULL DEFAULT 0,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

## 19.3 operations

```sql
CREATE TABLE operations (
    id TEXT PRIMARY KEY,
    file_id TEXT NOT NULL,
    operation_type TEXT NOT NULL,
    source_path TEXT,
    destination_path TEXT,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL,
    completed_at TEXT,
    undone_at TEXT,
    error_code TEXT,
    error_message TEXT,
    FOREIGN KEY(file_id) REFERENCES files(id)
);
```

## 19.4 batches

```sql
CREATE TABLE batches (
    id TEXT PRIMARY KEY,
    created_at TEXT NOT NULL,
    closed_at TEXT,
    status TEXT NOT NULL
);
```

## 19.5 batch_files

```sql
CREATE TABLE batch_files (
    batch_id TEXT NOT NULL,
    file_id TEXT NOT NULL,
    PRIMARY KEY(batch_id, file_id),
    FOREIGN KEY(batch_id) REFERENCES batches(id),
    FOREIGN KEY(file_id) REFERENCES files(id)
);
```

## 19.6 settings

```sql
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value_json TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

v0.2：

```sql
source_contexts
browser_downloads
```

---

# 20. 文件下载完成判断

这是 MVP 最关键的工程模块之一。

不能因为收到 `Create` 就认为文件完成。

## 20.1 临时文件过滤

默认忽略/观察：

```text
*.crdownload
*.part
*.tmp
*.download
```

注意：

- 临时扩展名可能 rename 为最终文件。
- rename event 必须被视为重要信号。
- 不得只依赖扩展名判断。

## 20.2 Stability Check

推荐算法：

```text
收到候选文件
↓
记录 size / modified time
↓
等待 300ms
↓
再次读取 metadata
↓
如果 size/mtime 变化：
    重置稳定计时器
↓
连续 N 次稳定
↓
尝试共享读取/open metadata
↓
READY
```

建议：

```text
check interval: 300ms
stable rounds: 2~3
max wait: 120s
```

大文件下载不能阻塞 watcher。

每个候选文件都应该是独立 async task / state。

## 20.3 超时

超过最大等待时间：

```text
status = Error / PendingRetry
```

后台低频重试。

不能删除或移动未确认完成的文件。

---

# 21. Event Debounce

文件系统可能产生：

```text
Create
Modify
Modify
Modify
Rename
Modify
```

必须归一化。

推荐键：

```text
normalized_path
```

未来可加入 file-id。

伪代码：

```rust
on_fs_event(event):
    key = normalize(event.path)

    pending[key].merge(event)
    pending[key].deadline = now + debounce_window

on_deadline(key):
    emit_candidate(pending.remove(key))
```

Debounce 推荐：

```text
150~500ms
```

与 Batch Window 分开。

Debounce 解决 OS 噪声。

Batch 解决 UX 多文件合并。

---

# 22. 批处理算法

Batch engine 接收：

```text
ReadyFileEvent
```

不是原始 FS event。

```rust
struct ReadyFileEvent {
    file_id: Uuid,
    path: PathBuf,
    ready_at: Instant,
    source_key: Option<String>,
}
```

算法：

```text
第一个 ReadyFile
→ 创建 open batch
→ 启动 2 秒 batch deadline

后续 ReadyFile
→ 若符合 merge policy
→ append
→ 可选择延长 deadline，但设置 max batch age

deadline 到
→ close batch
→ 通知 UI
```

必须设置：

```text
max_batch_age: 5s
max_batch_items: 100
```

防止持续下载导致永远不弹卡片。

---

# 23. 文件移动策略

Group 可有：

```text
destination_path = D:\Projects\ProjectA\Inbox
```

选择 Group 后：

```text
source:
C:\Users\User\Downloads\a.png

destination:
D:\Projects\ProjectA\Inbox\a.png
```

## 23.1 文件名冲突

MVP 默认：

```text
a.png
a (1).png
a (2).png
```

禁止 overwrite。

未来可提供：

- skip
- rename
- replace（高级选项）

MVP 永远不 replace。

## 23.2 跨磁盘

同卷：

```text
rename/move
```

跨卷可能变成：

```text
copy → fsync/verify → remove source
```

必须确保：

- copy 成功
- destination 可访问
- size 一致

然后才处理 source。

---

# 24. Undo 设计

Undo 必须基于 Operation Log，而不是 UI history。

示例：

```text
MOVE
source_path = Downloads\a.png
destination_path = D:\ProjectA\a.png
```

Undo：

```text
destination 存在？
source 不存在？
source parent 可写？
↓
move destination → source
↓
operation.undone_at = now
```

如果 source 已有同名：

不得覆盖。

显示冲突：

```text
Restore as:
a (restored).png
```

或让用户选择。

MVP 可以自动生成安全文件名。

---

# 25. Temporary 生命周期

Temporary 是核心产品能力之一。

用户选择：

```text
1 day
3 days
7 days
30 days
custom
```

MVP 默认：

```text
7 days
```

到期后：

```text
Temporary
↓
Expired
↓
CleanupReady
```

MVP 不自动永久删除。

用户可（完整状态转移见第 13 节，此处为落点摘要）：

```text
Keep 7 more days   → 状态回到 TEMPORARY，expires_at 重置
Move to Group      → 状态进入 ORGANIZING，成功后 ORGANIZED
Move to Recycle Bin→ 状态进入 TRASHED（终态，MVP 内不提供 Undo）
Ignore             → 状态不变，仍为 CLEANUP_READY，仅关闭本次提醒
```

未来可加入“自动进入系统回收站”，必须显式 opt-in。

---

# 26. Tauri IPC 设计

UI 不直接接触 DB 和文件系统。

所有操作走 typed command。

示例：

```rust
#[tauri::command]
async fn list_inbox(...)

#[tauri::command]
async fn list_groups(...)

#[tauri::command]
async fn assign_group(file_ids: Vec<Uuid>, group_id: Uuid)

#[tauri::command]
async fn mark_temporary(file_ids: Vec<Uuid>, ttl_days: u32)

#[tauri::command]
async fn mark_later(file_ids: Vec<Uuid>)

#[tauri::command]
async fn undo_operation(operation_id: Uuid)

#[tauri::command]
async fn open_file(file_id: Uuid)

#[tauri::command]
async fn reveal_in_explorer(file_id: Uuid)

#[tauri::command]
async fn update_settings(...)
```

Rust → UI events：

```text
file-ready
batch-ready
operation-completed
operation-failed
file-missing
settings-updated
```

Payload 必须版本化或至少强类型。

---

# 27. 前端状态原则

React 可以缓存视图状态，但不得成为业务 source of truth。

推荐：

- TanStack Query 或轻量自建 query layer
- Zustand 仅管理 UI ephemeral state

例如：

可以存在前端：

```text
selectedFileIds
sidebarCollapsed
floatingCardExpanded
```

不能只存在前端：

```text
file.status
operation.status
group assignment
expires_at
```

---

# 28. 系统托盘

托盘菜单：

```text
Open Download Inbox
Pause Watching
Inbox (6)
Temporary (12)
---
Start with Windows ✓
Settings
---
Quit
```

关闭主窗口：

默认：

```text
hide window
keep core running
```

Quit 才真正退出。

---

# 29. 开机启动

默认建议：

第一次安装：

```text
autostart = false
```

Onboarding 中明确询问：

> Start Download Inbox with Windows?

用户主动开启。

禁止偷偷注册开机启动。

---

# 30. 浏览器来源上下文（v0.2）

不是 MVP blocker。

目的：

不仅记录：

```text
Chrome
```

而是记录：

```text
domain
page title
source URL
download URL
browser download ID
timestamp
```

## 30.1 Extension

Chrome / Edge：

- Manifest V3
- TypeScript
- downloads permission
- nativeMessaging permission

监听下载：

```text
chrome.downloads.onCreated
chrome.downloads.onChanged
```

记录：

```ts
type DownloadContext = {
  browserDownloadId: number
  filename?: string
  url: string
  finalUrl?: string
  referrer?: string
  pageUrl?: string
  pageTitle?: string
  startedAt: string
}
```

## 30.2 Native Messaging

推荐 Chrome Native Messaging。

官方协议核心：

```text
Extension
↓
runtime.connectNative / sendNativeMessage
↓
Native Messaging Host
↓
stdin/stdout
↓
length-prefixed UTF-8 JSON
```

Windows 安装器负责注册 Host manifest。

Native host 不应该和主 UI 强耦合。

推荐：

```text
browser extension
→ native-messaging-host.exe
→ local IPC
→ desktop core
```

或者第一版 v0.2 简化：

```text
native host
→ 写入 SQLite / named pipe
```

最终应统一由 Core 接收。

---

# 31. 来源匹配

浏览器事件和文件系统事件可能不同步。

不要假设：

```text
browser event == fs event
```

匹配评分可使用确定性特征：

```text
filename
path
timestamp distance
size
download ID metadata
```

禁止用 AI。

匹配结果：

```text
Exact
Probable
Unknown
```

UI 对 Unknown 不显示错误来源。

---

# 32. 性能目标

这是项目卖点，需要实际 benchmark 验证。

目标值，不是承诺值：

## Idle

```text
CPU: ~0%
Disk IO: ~0
Network: 0
RAM:
  Stretch goal < 20 MB
  Acceptable MVP < 35 MB
```

注意：

Tauri WebView 主窗口打开时 RAM 不计入“后台 idle core”目标。

Benchmark 应区分：

```text
tray only / windows hidden
main UI open
batch processing
```

## Event load

目标：

```text
100 files burst:
- 不丢事件
- 不弹 100 个窗口
- UI 可响应

1,000 synthetic file events:
- queue 不崩
- DB 无损
```

---

# 33. 性能测量

建立：

```text
docs/performance.md
```

至少记录：

- OS
- CPU
- RAM
- Rust version
- build type
- idle RSS
- event throughput
- 100 / 1,000 file benchmark
- SQLite write latency

只报告实测。

README 不允许伪造：

```text
5 MB RAM
0.01% CPU
```

必须有可重复测试方法。

---

# 34. 安全与隐私

默认：

```text
No account
No cloud
No telemetry
No AI
No file upload
```

数据库可能包含：

- 文件名
- 本地路径
- 来源 URL（v0.2）
- 网页标题

所以：

- DB 仅保存在用户本机 app data。
- Debug log 默认不得完整记录敏感 URL query。
- 不记录文件内容。
- 不读取文件内容，除非未来功能明确要求。
- 不扫描 Downloads 之外目录，除非用户主动添加 monitored folder。

---

# 35. 文件安全规则

任何代码修改文件前必须满足：

```text
1. path 已规范化
2. path 属于允许操作范围
3. source 存在
4. destination 未覆盖已有文件
5. DB operation 已创建
6. 操作结果被验证
```

禁止：

```rust
std::fs::remove_file(user_path)
```

散落在业务代码中。

只能：

```text
file-operations::trash()
file-operations::move_safe()
file-operations::restore()
```

---

# 36. 错误模型

错误必须机器可读。

示例：

```rust
enum AppErrorCode {
    PermissionDenied,
    SourceMissing,
    DestinationExists,
    FileLocked,
    InvalidPath,
    DownloadNotComplete,
    DatabaseError,
    WatcherError,
    CrossVolumeMoveFailed,
    UndoConflict,
}
```

UI 根据 code 展示可理解信息。

禁止把：

```text
Os error 32
```

直接扔给用户。

---

# 37. Logging

使用：

```text
tracing
tracing-subscriber
```

日志级别：

```text
ERROR
WARN
INFO
DEBUG
TRACE
```

Release 默认：

```text
INFO or WARN
```

Debug log 不写文件正文。

路径日志可在 debug mode 保留，但 issue bundle 要允许脱敏。

---

# 38. SQLite 策略

建议：

```text
WAL mode
foreign_keys = ON
busy_timeout
```

数据库写入尽量短事务。

不要长时间持有写锁。

关键文件操作模式：

```text
BEGIN
insert operation pending
COMMIT

execute filesystem operation

BEGIN
mark operation completed
update file state
COMMIT
```

注意：

文件系统与 SQLite 无法形成真正 ACID transaction。

因此必须支持启动时 reconciliation。

---

# 39. Crash Recovery / Reconciliation

程序可能在：

```text
DB 写 pending
↓
文件 move 成功
↓
程序 crash
↓
DB 未 mark completed
```

启动时扫描：

```text
operations.status = pending
```

根据：

```text
source exists?
destination exists?
```

恢复状态。

示例：

```text
source missing + destination exists
→ 推断 move 可能已完成
→ verify
→ mark completed_recovered
```

这一点非常适合作为面试工程亮点。

---

# 40. 启动流程

```text
app start
↓
init logging
↓
resolve app directories
↓
open SQLite
↓
run migrations
↓
reconcile incomplete operations
↓
load settings
↓
start watcher
↓
create tray
↓
UI window hidden / onboarding
```

Watcher 失败不能导致 DB 损坏。

---

# 41. Onboarding

第一次启动：

### Step 1

```text
Welcome
Your downloads should not become a junk drawer.
```

### Step 2

自动检测：

```text
C:\Users\<user>\Downloads
```

让用户确认。

### Step 3

创建 2~3 个示例 Group，或让用户创建。

建议默认：

```text
Projects
Keep
```

Temporary 是系统状态，不必是 Group。

### Step 4

询问：

```text
Start with Windows?
```

完成。

---

# 42. Settings

MVP：

```text
Watched folder
Floating card enabled
Card timeout
Batch window
Default temporary duration
Start with Windows
Pause watcher
Show notifications
Database location (read-only)
```

后续：

```text
multiple watched folders
quiet hours
fullscreen suppression
browser source integration
```

---

# 43. Quiet / Do Not Disturb

MVP 可简化：

检测系统无法可靠实现时，先提供手动：

```text
Pause for 1 hour
Pause until tomorrow
Pause watching
```

v0.2 再考虑：

- full-screen detection
- focus assist
- presentation mode

---

# 44. 测试策略

测试分四层。

## Unit

必须覆盖：

- debounce merge
- batch grouping
- filename conflict resolution
- TTL calculation
- state transitions
- path normalization

## Integration

使用 tempdir：

```text
create file
modify file
rename temp → final
move
undo
collision
```

## Database

每次 migration 测试：

```text
fresh DB
upgrade previous schema
FK integrity
```

## E2E

Windows CI 或本地：

```text
start app core
simulate download
assert InboxItem
assign group
assert file moved
undo
assert restored
```

---

# 45. 必须测试的边界场景

至少包含：

1. 0 字节文件
2. 1KB 文件
3. 5GB 文件（可 mock metadata）
4. Unicode 文件名
5. 中文文件名
6. Emoji 文件名
7. 超长路径
8. 同名文件
9. 文件正在被占用
10. 下载后立即删除
11. 下载后立即 rename
12. `.crdownload → final`
13. 100 文件同时完成
14. 目标磁盘离线
15. 目标目录无权限
16. 跨卷移动
17. 程序在 move 中 crash
18. Undo 时原路径已有同名文件
19. Downloads 在 OneDrive 下
20. 网络盘路径（MVP 可声明 best-effort）

---

# 46. CI

GitHub Actions：

## PR

```text
cargo fmt --check
cargo clippy -- -D warnings
cargo test
frontend lint
frontend typecheck
frontend test
```

## Release

```text
Windows build
artifact upload
checksum
GitHub Release
```

后期：

- code signing
- macOS build
- Linux build

---

# 47. 代码质量规则

Rust：

- 禁止 `unwrap()` 出现在用户可触发的正常路径。
- 错误必须向上传递或显式处理。
- Domain 层禁止依赖 Tauri。
- IO 操作不得阻塞 async runtime 主线程。
- 大文件 copy/hash 需要 `spawn_blocking` 或专用线程。
- 对共享状态优先消息传递，不滥用全局 Mutex。
- 所有状态枚举禁止 magic string 散落。

TypeScript：

- `strict: true`
- API 类型统一生成/共享，避免手写漂移。
- 禁止 `any` 作为逃生口。
- 业务操作只能调用 Tauri command layer。
- Components 不直接拼 filesystem path。

---

# 48. 依赖控制

这个项目的主题之一是轻量。

新增依赖必须回答：

```text
1. 为什么需要？
2. 标准库能否完成？
3. 对 binary size 的影响？
4. 对 idle memory 的影响？
5. 是否引入 runtime / background thread？
```

禁止为了一个小 utility 引入大型 framework。

---

# 49. v0.1 Milestones

## M0 — Skeleton

完成：

- Tauri 2 + React
- Rust workspace
- SQLite migration
- tray
- basic CI

验收：

```text
app opens
window closes to tray
DB initialized
CI green
```

## M1 — Watcher

完成：

- monitor Downloads
- normalize events
- completion detection
- create file record

验收：

```text
下载文件完成后 1~3 秒内进入 READY/PENDING
.crdownload 不进入最终 Inbox
```

## M2 — Floating Card

完成：

- non-focus-stealing card
- single file
- Later

验收：

```text
正在 VS Code 输入时下载文件
→ 卡片出现
→ VS Code 输入焦点不丢
```

## M3 — Groups + Move

完成：

- create group
- destination path
- assign group
- safe move
- collision rename

验收：

```text
选择 group
→ 文件移动成功
→ DB 与实际路径一致
```

## M4 — Undo

完成：

- operation log
- undo
- conflict safe restore

验收：

```text
move → undo → 文件恢复
```

## M5 — Batch

完成：

- batch engine
- batch floating card
- batch assign

验收：

```text
一次下载 20 张图
→ 不产生 20 个卡片
→ 1 个 batch card
```

## M6 — Temporary

完成：

- TTL
- expired
- cleanup queue
- recycle bin action

## M7 — Polish / OSS

完成：

- README
- GIF
- docs
- benchmarks
- Release v0.1.0

---

# 50. v0.2 Roadmap

优先：

1. Chrome / Edge Extension
2. Native Messaging
3. Source URL / Domain / Page Title
4. Multi-monitor polish
5. Keyboard shortcuts
6. Custom watched folders
7. Better quiet mode

---

# 51. v0.3 Roadmap

可选：

- Rule learning，但不是 AI
- 用户显式创建规则
- Auto-route trusted sources
- Tags
- Search
- File preview
- Duplicate detection
- Recycle Bin automation
- macOS

注意：

规则功能只能作为增强，不能让产品退化成另一个传统 auto organizer。

---

# 52. Duplicate Detection（后续）

不是 MVP blocker。

若实现：

优先两级：

```text
size
↓
BLAKE3 / SHA-256
```

大文件 Hash：

- lazy
- background
- low priority
- cancelable

不要因为 Hash 阻塞新文件弹窗。

---

# 53. UI 视觉方向

目标：

- Native utility
- 极简
- 低视觉噪音
- Windows 11 友好
- 不做 Electron Dashboard 风格

参考精神：

- Raycast
- Linear
- Arc 小型浮层
- Windows 11 toast

但必须形成自己的样式。

设计关键词：

```text
compact
quiet
fast
local
utility
```

---

# 54. Demo 设计

GitHub 最关键 Demo：

### Demo A — 主流程

15 秒：

```text
打开浏览器
↓
下载一张图片
↓
卡片出现
↓
选择 “Design”
↓
打开 Explorer
↓
文件已进入 Design
↓
点击 Undo
↓
文件回 Downloads
```

### Demo B — Batch

```text
下载 10 张图片
↓
只出现一个 “10 new files” 卡片
↓
全部 Temporary
```

### Demo C — Lightweight

Task Manager：

```text
window hidden
CPU near idle
RAM measured
```

只展示真实数据。

---

# 55. README 第一屏建议

```md
# Download Inbox

> Your Downloads folder isn't a filing cabinet.

[demo gif]

A tiny, local-first Windows utility that asks where a download belongs while you still remember.

- No AI
- No cloud
- No account
- No background scanning
- Undo everything
```

然后：

```text
Why?
How it works
Demo
Features
Performance
Architecture
Roadmap
Install
Contributing
```

README 不应该先讲：

```text
This app is built with Rust and Tauri...
```

先讲痛点。

---

# 56. 开源策略

License 建议：

```text
MIT
```

或：

```text
Apache-2.0
```

如果目标是最大传播，MIT 更简单。

必须有：

```text
CONTRIBUTING.md
CODE_OF_CONDUCT.md（可选）
SECURITY.md
Issue template
Feature request template
Bug template
```

设置标签：

```text
good first issue
help wanted
bug
enhancement
windows
ux
performance
```

---

# 57. Star 获取策略（产品内生）

不依赖刷榜。

项目本身应具备：

- 一眼能懂
- GIF 可传播
- 真实痛点
- 单文件安装
- 不登录
- 本地运行
- Rust/Tauri 技术标签
- 性能数据
- 有明确差异
- README 好看
- 开源 roadmap 清楚

推荐发布渠道：

- GitHub
- Hacker News / Show HN
- Reddit：
  - r/rust
  - r/opensource
  - r/windows
  - r/productivity
- X
- V2EX
- 掘金
- 少数派社区（视发布规则）
- Product Hunt（后期）

---

# 58. 与类似项目的差异化

本项目不要宣传：

> 自动帮你分类文件。

而是：

> **我们不猜。我们在你还记得的时候问一次。**

核心区别：

| 传统 Organizer | 本项目 |
|---|---|
| 事后整理 | 文件产生时整理 |
| 规则猜测 | 用户 1 秒确认 |
| 按扩展名 | 按真实项目上下文 |
| 后台自动搬运 | Human-in-the-loop |
| 容易误分类 | 用户明确决策 |
| 复杂规则 | 低交互成本 |

---

# 59. 面试讲解框架

推荐 3 分钟：

## 1. Problem

Downloads 不是存档系统。

用户下载时知道文件用途，但几天后上下文消失。

## 2. Product insight

不是事后自动分类，而是：

```text
Capture context at creation time.
```

## 3. Architecture

```text
notify
→ debounce
→ completion detector
→ batch engine
→ Tauri floating window
→ user action
→ safe file operation
→ SQLite operation log
```

## 4. Difficult engineering problems

重点讲：

- `.crdownload` 完成判断
- OS 重复文件事件
- 不抢焦点
- 100 文件 batching
- cross-volume move
- crash recovery
- Undo
- low idle memory

## 5. Tradeoff

为什么没有 AI：

> 用户刚下载时拥有比模型更准确的上下文，产品问题不是“如何猜”，而是“如何在最低打扰下捕获这个上下文”。

这是一个非常好的产品/工程取舍案例。

---

# 60. ADR — Architecture Decisions

## ADR-001: Rust 常驻核心

Decision：

```text
Rust
```

Reason：

- 低后台内存
- OS integration
- 性能
- 单 binary
- 工程展示价值

Reject：

- Python 常驻
- Node backend
- Electron main process

---

## ADR-002: 不使用 AI

Decision：

```text
Human-in-the-loop
```

Reason：

用户在文件产生时拥有准确上下文。

避免：

- 模型错误
- 隐私问题
- 云调用
- runtime
- 复杂度

---

## ADR-003: Tauri 2

Reason：

- Rust core
- Web frontend development speed
- tray
- desktop windows
- packaging
- 比 Electron 更符合轻量目标

---

## ADR-004: SQLite

Reason：

- local-first
- no service
- portable
- enough scale
- transaction/history

---

## ADR-005: Event-driven watcher

Decision：

```text
notify / native FS events
```

Reject：

```text
while true scan Downloads
```

Reason：

- CPU
- disk
- latency
- battery
- elegance

---

## ADR-006: Operation Log for Undo

所有文件移动都成为可恢复 Operation。

Reason：

文件工具必须 safe-by-default。

---

# 61. Agent Task Template

Agent 每次实现任务时应输出：

```text
Goal
Files changed
Design decision
Implementation
Tests
Acceptance result
Known limitations
```

示例：

```md
## Goal
Implement download completion detection.

## Scope
- crates/download-detector
- no UI changes

## Acceptance
- .crdownload ignored
- rename final detected
- stable file emits Ready
- test coverage included
```

---

# 62. Agent 开发顺序

严格优先：

```text
1. Domain model
2. SQLite/migrations
3. Watcher
4. Completion detector
5. Event engine
6. Inbox service
7. Tauri commands
8. Floating UI
9. File operations
10. Undo
11. Batch
12. Temporary
13. Browser extension
```

不要先做漂亮 UI 再补 Core。

也不要先做 Browser Extension。

---

# 63. Definition of Done

一个功能只有满足全部条件才算完成：

- 编译通过
- lint 通过
- 单测通过
- 关键路径 integration test
- 没有新增不必要依赖
- 错误可处理
- 不破坏 Undo
- 不破坏 Local-first
- 文档更新
- Acceptance Criteria 通过

---

# 64. MVP 最终验收

## Functional

- [ ] 自动检测默认 Downloads
- [ ] 新下载完成后创建 Inbox item
- [ ] 临时下载文件不会误弹
- [ ] Floating Card 不抢焦点
- [ ] 可选择 Group
- [ ] 可选择 Temporary
- [ ] 可选择 Later
- [ ] 文件移动安全
- [ ] 同名不覆盖
- [ ] 操作可 Undo
- [ ] 20 文件下载自动 batch
- [ ] 主窗口可查看 pending
- [ ] 主窗口可查看 history
- [ ] 托盘工作正常
- [ ] 可暂停 watcher
- [ ] 可开关 Windows autostart

## Reliability

- [ ] Crash 不导致文件消失
- [ ] DB/FS 状态可 reconciliation
- [ ] 文件锁定时不强行移动
- [ ] 跨盘移动失败时源文件保留
- [ ] Undo 冲突不覆盖文件

## Performance

- [ ] 隐藏 UI 时 CPU 基本空闲
- [ ] 无目录轮询
- [ ] 无网络请求
- [ ] 100 文件 burst 不崩
- [ ] benchmark 数据已写入 docs

## OSS

- [ ] README
- [ ] GIF
- [ ] LICENSE
- [ ] CONTRIBUTING
- [ ] CI
- [ ] Release binary
- [ ] Architecture docs
- [ ] Roadmap

---

# 65. 后续可以做但不要污染 MVP 的想法

候选：

- Browser source context
- Tags
- Search
- Duplicate detection
- Smart deterministic rules
- Per-source preferred groups
- Keyboard-first popup
- Quick command palette
- Custom destination templates
- Portable mode
- Multi-folder watch
- macOS/Linux
- Plugin API

明确不优先：

- AI
- LLM
- OCR
- Cloud sync
- Team collaboration

---

# 66. 最小可行开发切片

如果 Agent 要立刻开始，第一条 vertical slice：

```text
Downloads 创建一个完整 txt 文件
↓
Watcher 检测
↓
Completion Detector 确认 READY
↓
SQLite 建立 file
↓
Tauri emit `file-ready`
↓
Floating Card 显示文件名
↓
点 Later
↓
DB status = Later
```

这个切片完成后再做 Move。

不要第一天同时实现：

- group
- undo
- browser extension
- batch
- temporary

---

# 67. 推荐第一批 GitHub Issues

```text
#1 Initialize Tauri 2 + Rust workspace
#2 Add SQLite migrations
#3 Implement normalized file watcher
#4 Implement download completion detector
#5 Build event debounce engine
#6 Create Inbox domain model
#7 Implement non-focus-stealing floating card
#8 Add group CRUD
#9 Implement safe move operation
#10 Implement operation log
#11 Add Undo
#12 Add batch aggregation
#13 Add temporary lifecycle
#14 Add system tray
#15 Add autostart settings
#16 Add performance benchmark harness
#17 Record demo GIF
```

适合 `good first issue`：

```text
Add file-size formatter
Add icon by file extension
Add empty Inbox state
Add localization scaffold
Add keyboard shortcut
Improve filename collision test cases
```

---

# 68. 参考技术资料

Tauri 2：

- System Tray: https://v2.tauri.app/learn/system-tray/
- Autostart: https://v2.tauri.app/plugin/autostart/

Rust notify：

- https://docs.rs/notify/latest/notify/

SQLx SQLite：

- https://docs.rs/sqlx/latest/sqlx/sqlite/

Chrome Native Messaging：

- https://developer.chrome.com/docs/extensions/develop/concepts/native-messaging

这些资料只作为实现参考；实际开发使用当时稳定版本并锁定依赖。

---

# 69. 最终产品哲学

这个项目最重要的不是技术栈，而是这个判断：

```text
传统思路：
文件乱了 → 再整理

本项目：
文件刚产生 → 趁用户还记得 → 轻问一次 → 以后不乱
```

所以任何功能设计都必须回答：

> 它有没有让“文件产生后的几秒钟”更轻、更快、更安全？

如果答案是否定的，就不应该进入 MVP。

---

# 70. Agent 最终约束摘要

```yaml
project:
  type: desktop_utility
  purpose:
    - open_source
    - portfolio
    - interview_project
  primary_os: windows
  local_first: true
  ai: false
  cloud: false
  account: false

stack:
  desktop: tauri_2
  frontend:
    - react
    - typescript
  core: rust
  async: tokio
  watcher: notify
  database: sqlite
  db_access: sqlx
  logging: tracing
  browser_extension:
    phase: v0.2
    manifest: v3
    transport: chrome_native_messaging

architecture:
  event_driven: true
  polling_downloads: forbidden
  ui_direct_filesystem_access: forbidden
  safe_file_operations_only: true
  undo_required: true
  operation_log_required: true

mvp:
  - download_watch
  - completion_detection
  - floating_card
  - groups
  - temporary
  - later
  - batching
  - sqlite_history
  - undo
  - tray
  - autostart_toggle

non_goals:
  - ai
  - llm
  - cloud_sync
  - auth
  - vector_database
  - electron
  - python_backend
  - automatic_permanent_delete

performance:
  idle_cpu: near_zero
  network_idle: zero
  target_idle_ram_mb: "<35 MVP, <20 stretch"
  background_directory_polling: false

safety:
  overwrite_existing_file: false
  permanent_delete_mvp: false
  crash_recovery: required
  cross_volume_failure_preserves_source: true
```

---

**End of specification.**
