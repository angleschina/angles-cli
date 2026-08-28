<div align="center">

<p align="center"><img src="assets/angles-rainbow.svg" width="140" alt="Angles"></p>

<h1 align="center">Angles Code CLI</h1>

### 1.6 MB 的智能编码工具 —— 单一静态 Rust 二进制，零运行时依赖

<p>
  <a href="./README.md">English</a> ·
  <strong>简体中文</strong>
</p>

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Platforms](https://img.shields.io/badge/Platforms-Linux%20%7C%20Windows%20%7C%20macOS-lightgrey.svg)](https://github.com/ZSJ305/angles-cli/releases)

<a href="https://github.com/ZSJ305/angles-cli/releases"><img src="assets/github-badge.svg" alt="下载 · GitHub"></a>

**30+ 内置工具 · 11 家模型供应商 · 5 平台预编译 · MIT 开源**

<p>
  <a href="https://zsj305.github.io/angles-cli/">官网</a> ·
  <a href="https://zsj305.github.io/angles-cli/docs.html">文档</a> ·
  <a href="https://zsj305.github.io/angles-cli/tools.html">工具参考</a> ·
  <a href="https://www.npmjs.com/package/@angleschina/angles">npm</a> ·
  <a href="https://github.com/ZSJ305/angles-cli/releases">Releases</a> ·
  <a href="https://github.com/ZSJ305/angles-cli/issues">Issues</a>
</p>

```bash
npm i -g @angleschina/angles && angles install
```

</div>

Angles Code CLI 是一个终端原生的智能编码助手，编译为**单个静态 Rust 二进制（约 1.6 MB）**。它不依赖 Node、不依赖 Python、不依赖任何运行时或编译器工具链——一条命令装好即可开始工作。代理（agent）通过 **30+ 个内置的 `angles-*` 工具**直接操作文件、目录、终端、Git 与网络，并可在 **11 家模型供应商**之间任意切换。自 v0.4.1 起，Angles 在执行前会先生成**AI 操作计划**：列出「读取… 安装… 创建… 编辑… 运行…」等高层次步骤，让你始终清楚它接下来要做什么。配置全部保存在本地 `~/.angles/` 下，API Key 永不经由任何中继服务器。

| | |
|---|---|
| 📦 二进制大小 | **1.6 MB**（musl 静态 / MSVC） |
| 🧩 内置工具 | **30+**（文件 / 目录 / 终端 / Git / 网络） |
| 🤖 模型供应商 | **11 家**（OpenAI / Claude / Gemini / DeepSeek / Grok / Qwen / GLM / Kimi …） |
| 🖥️ 预编译平台 | **5 个**（Linux ARM64/x64 · macOS ARM64/x64 · Windows x64） |
| ⚡ 运行时依赖 | **0**（纯 Rust，零 Node / 零 Python / 零 libc 动态链接） |
| 🔓 许可证 | **MIT** |

---

## 为什么做成单二进制？

多数智能编码工具以 Node / Python 包形式分发，附带成百上千个传递依赖；或做成体积庞大的多层安装器。这在多数情况下没问题——直到你需要目标是一台 ARM64 单板机、一个无 root 的容器，或是 iOS 上的 iSH。

Angles 编译成**零运行时依赖**的单个静态二进制。分发问题被压缩成「下载一个 ≤ 1.6 MB 的文件并 chmod +x」。在装工具链不现实受限环境里这一点尤其关键：

| 限制 | 常规工具链 | Angles |
|---|---|---|
| 运行时 / 解释器 | 需要 Node 或 Python | **无**（原生静态二进制） |
| 安装占用 | 数十 MB + 依赖树 | **≤ 1.6 MB** |
| 从源码构建 | 往往必需 | **可选**（有预编译二进制） |
| ARM64 SBC / 无 root / iSH | 常常失败 | **可用** |

---

## 架构总览

```
angles-cli/
├── src/
│   ├── main.rs          # 入口与命令路由
│   ├── cli.rs           # Clap CLI 定义与帮助
│   ├── config.rs        # 配置加载 / 保存 / 展示
│   ├── provider.rs      # 11 家供应商注册表与 base URL
│   ├── gateway.rs       # TUI 设置向导（dialoguer）
│   ├── instructions.rs  # 系统提示模板渲染（handlebars）
│   ├── api.rs           # API 客户端（OpenAI/Anthropic/Gemini）+ 流式 + 工具循环
│   ├── search.rs        # 网页搜索 URL 构建
│   ├── server.rs        # 本地 HTTP 网关（axum）—— `angles serve`
│   └── tools.rs         # 30+ 个 angles-* 工具实现 + `doctor`
├── instructions.txt     # 系统提示模板（13 KB，{{variable}} 注入）
├── AGENTS.md            # 代理工具参考
├── providers.toml       # 供应商数据源
├── gateway-flow.md      # 向导流程说明
├── docs/                # GitHub Pages 站点（index/docs/tools/faq.html）
├── Cargo.toml
├── Makefile
├── Cross.toml
└── .github/workflows/release.yml
```

对话循环运行在 `api.rs` 中：一个带流式的 OpenAI / Anthropic Messages / Gemini 原生客户端，并以工具调用循环为支撑，将 `angles-*` 命令解析到 `tools.rs` 中对应的实现。

---

## 关键设计决策

### 1. 工具驱动的代理循环

Angles 的每一项能力都是一等公民的 `angles-` 命令。代理被绑定到一套经过策划、行为确定（deterministic）的工具集上，而不是让模型自由发挥 shell 单行命令：

- **文件**：`angles-createfile/writefile/appendfile`、`angles-readfile`、`angles-replace/replaceall`、`angles-grep`、`angles-head/tail` …
- **终端**：`angles-run`（前台）、`angles-runbg`（后台，返回 PID）、`angles-kill`
- **Git**：`angles-gitinit/gitcommit/gitlog/gitdiff/gitbranch`
- **网络**：`angles-fetch`、`angles-websearch`
- **目录 / 管理**：`angles-ls/tree/cd/pwd`、`angles-fileinfo`、`angles-*dir/copy`

**安全模型**：读取操作无需批准；写入遵循配置的审批策略；删除及危险操作始终需要用户确认（见 [AGENTS.md](AGENTS.md)）。

### 2. 多供应商，统一通信协议

供应商注册表（`provider.rs` + `providers.toml`）把 11 家供应商各自映射到其原生线上协议——OpenAI Chat Completions、Anthropic Messages 或 Gemini Native——并对流式与工具调用路径做归一化，使代码库其余部分与具体供应商解耦。

### 3. AI 操作计划（v0.4.1+）

动手之前，Angles 会根据系统提示渲染一份高层次计划并在终端中给出：*读取… 安装… 创建… 编辑… 运行…*。每一步都可见、可审计；代理不会静默操作。

### 4. 本地 HTTP 网关（v0.2+）

`angles serve` 会启动一个内嵌的 axum 服务器，监听 `http://127.0.0.1:8080`——一个可在浏览器里改配置、切换供应商、测试对话的 Web 控制台。REST 接口：`/health`、`/api/config`、`/api/providers`、`/api/chat`。

### 5. 隐私优先的配置

所有内容都保存在 `~/.angles/config.json`。API Key 可从本地文件读取，或通过 `ANGLES_API_KEY` 环境变量设置——绝不发送到中继服务器。

---

## 配置

保存在 `~/.angles/config.json`：

```json
{
  "language": "zh-CN",
  "provider": "glm",
  "base_url": "https://api.siliconflow.cn/v1",
  "wire_api": "chat",
  "model": "zai-org/GLM-5.2",
  "api_key": "",
  "max_tokens": 16384,
  "daily_token_budget": 1000000,
  "agent_persona": "你是一个专业、高效的编码助手。",
  "search_engine": "bing",
  "search_engine_url": "",
  "approval_policy": "untrusted"
}
```

API Key 也可通过 `ANGLES_API_KEY` 环境变量提供。

---

## 安装

以下四种方式安装的是**同一个**二进制：

<table>
<tr>
<td>

**npm（推荐）**

```bash
npm i -g @angleschina/angles && angles install
```

国内默认走 npmmirror 镜像，下载最快。

</td>
<td>

**curl — Linux / macOS / WSL2**

```bash
curl -fsSL https://zsj305.github.io/angles-cli/install.sh | bash
```

</td>
</tr>
<tr>
<td>

**PowerShell — Windows**

```powershell
irm https://zsj305.github.io/angles-cli/install.ps1 | iex
```

</td>
<td>

**wget**

```bash
wget -qO- https://zsj305.github.io/angles-cli/install.sh | bash
```

</td>
</tr>
</table>

当有预编译二进制可用时，安装器会**完全跳过 Rust / 编译器工具链**，在几秒内下载约 1.6 MB 的二进制文件完成安装。在 iSH、树莓派、Alpine 以及无 root 环境下均可直接使用。

---

## 快速上手

```bash
# 首次配置（5 步 TUI 向导）
angles gateway

# 交互式会话（默认模式）
angles

# 一次性非交互执行
angles exec "写一个 Python HTTP 服务器"

# 只生成 AI 操作计划而不执行（v0.4.1+）
angles plan "给这个 Express 项目加 JWT 鉴权，并写测试"

# 启动本地 HTTP 网关（v0.2+）—— http://127.0.0.1:8080
angles serve

# 查看配置 / 诊断安装 / 帮助
angles config
angles doctor
angles help
```

---

## 支持的供应商

| Provider | API Host | 通信协议 |
|---|---|---|
| OpenAI | api.openai.com | OpenAI Chat Completions |
| Claude (Anthropic) | api.anthropic.com | Anthropic Messages API |
| Gemini (Google) | generativelanguage.googleapis.com | Gemini Native API |
| DeepSeek | api.deepseek.com | OpenAI Chat Completions |
| Grok (xAI) | api.x.ai | OpenAI Chat Completions |
| MiniMax | api.minimax.chat | OpenAI Chat Completions |
| OpenRouter | openrouter.ai | OpenAI Chat Completions |
| 通义千问 Qwen | dashscope.aliyuncs.com | OpenAI Chat Completions |
| 智谱 GLM | api.siliconflow.cn | OpenAI Chat Completions |
| Kimi (Moonshot) | api.moonshot.cn | OpenAI Chat Completions |
| 自定义 Custom | 用户自定 | OpenAI / Anthropic / Gemini |

---

## 从源码构建

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 克隆并构建
git clone https://github.com/ZSJ305/angles-cli.git
cd angles-cli
cargo build --release

# 安装
cp target/release/angles ~/.local/bin/
```

### 交叉编译

```bash
make setup-arm64 && make arm64   # Linux ARM64
make setup-x64 && make x64       # Linux x64
make macos-arm64                 # macOS ARM64（仅 macOS）
```

5 个平台的预编译二进制由 GitHub Actions 在每次打 tag 时自动产出——见 [Releases](https://github.com/ZSJ305/angles-cli/releases)。

---

## 许可证

**MIT** —— 见 [LICENSE](LICENSE)。
