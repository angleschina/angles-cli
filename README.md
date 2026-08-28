<div align="center">

<p align="center"><img src="assets/angles-rainbow.svg" width="140" alt="Angles"></p>

<h1 align="center">Angles Code CLI</h1>

### A 1.6 MB agentic coding tool — single static Rust binary, zero runtime dependencies

<p>
  <strong>English</strong> ·
  <a href="./README_zh.md">简体中文</a>
</p>

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Platforms](https://img.shields.io/badge/Platforms-Linux%20%7C%20Windows%20%7C%20macOS-lightgrey.svg)](https://github.com/ZSJ305/angles-cli/releases)

<a href="https://github.com/ZSJ305/angles-cli/releases"><img src="assets/github-badge.svg" alt="Download on GitHub"></a>

**30+ built-in tools · 11 model providers · 5 platforms prebuilt · MIT**

<p>
  <a href="https://zsj305.github.io/angles-cli/">Website</a> ·
  <a href="https://zsj305.github.io/angles-cli/docs.html">Documentation</a> ·
  <a href="https://zsj305.github.io/angles-cli/tools.html">Tool Reference</a> ·
  <a href="https://www.npmjs.com/package/@angleschina/angles">npm</a> ·
  <a href="https://github.com/ZSJ305/angles-cli/releases">Releases</a> ·
  <a href="https://github.com/ZSJ305/angles-cli/issues">Issues</a>
</p>

```bash
npm i -g @angleschina/angles && angles install
```

</div>

Angles Code CLI is a terminal-native agentic coding assistant compiled into a **single static Rust binary (~1.6 MB)**. It needs no Node, no Python, no runtime, and no compiler toolchain — install in one command and start working. The agent drives **30+ built-in `angles-*` tools** to operate files, directories, the terminal, Git, and the web, switching between **11 model providers** at will. Since v0.4.1 it emits **AI-generated operation plans** before executing: Angles lists high-level steps ("read… install… create… edit… run…") so you always know what it is about to do. Config lives locally under `~/.angles/`; API keys are never routed through a relay server.

| | |
|---|---|
| 📦 Binary size | **1.6 MB** (musl static / MSVC) |
| 🧩 Built-in tools | **30+** (file / dir / terminal / Git / web) |
| 🤖 Providers | **11** (OpenAI / Claude / Gemini / DeepSeek / Grok / Qwen / GLM / Kimi …) |
| 🖥️ Prebuilt platforms | **5** (Linux ARM64/x64 · macOS ARM64/x64 · Windows x64) |
| ⚡ Runtime dependencies | **0** (pure Rust, no Node / no Python / no libc dynamic linking) |
| 🔓 License | **MIT** |

---

## Why a single binary?

Most agentic coding tools ship as Node/Python packages pulling hundreds of transitive dependencies, or as large multi-layer installers. That works — until you need to reach an ARM64 SBC, a rootless container, or run inside iSH on iOS.

Angles compiles to one static binary with **zero runtime dependencies**. The distribution problem collapses to "download one file ≤ 1.6 MB and chmod +x". This matters on constrained environments where installing a toolchain is impractical:

| Constraint | Conventional toolchain | Angles |
|---|---|---|
| Runtime / interpreter required | Node or Python | **None** (native static binary) |
| Install footprint | tens of MB + dependency tree | **≤ 1.6 MB** |
| Toolchain to build from source | often required | **optional** (prebuilt binaries exist) |
| ARM64 SBC / rootless / iSH | often fails | **works** |

---

## Architecture overview

```
angles-cli/
├── src/
│   ├── main.rs          # Entry point & command routing
│   ├── cli.rs           # Clap CLI definitions & help
│   ├── config.rs        # Config load/save/display
│   ├── provider.rs      # 11-provider registry with base URLs
│   ├── gateway.rs       # TUI setup wizard (dialoguer)
│   ├── instructions.rs  # System-prompt template rendering (handlebars)
│   ├── api.rs           # API client (OpenAI/Anthropic/Gemini) + streaming + tool loop
│   ├── search.rs        # Web search URL builder
│   ├── server.rs        # Local HTTP gateway (axum) — `angles serve`
│   └── tools.rs         # 30+ angles-* tool implementations + `doctor`
├── instructions.txt     # System-prompt template (13 KB, {{variable}} injection)
├── AGENTS.md            # Agent tool reference
├── providers.toml       # Provider data source
├── gateway-flow.md      # TUI wizard flow spec
├── docs/                # GitHub Pages site (index/docs/tools/faq.html)
├── Cargo.toml
├── Makefile
├── Cross.toml
└── .github/workflows/release.yml
```

The conversation loop runs in `api.rs`: an OpenAI / Anthropic Messages / Gemini-native client with streaming, backed by a tool-calling loop that resolves `angles-*` commands to the corresponding implementations in `tools.rs`.

---

## Key design decisions

### 1. Tool-driven agent loop

Every capability Angles exposes is a first-class `angles-` command. Rather than letting the model improvise shell one-liners, the agent is bound to a curated, deterministic tool set:

- **File**: `angles-createfile/writefile/appendfile`, `angles-readfile`, `angles-replace/replaceall`, `angles-grep`, `angles-head/tail` …
- **Terminal**: `angles-run` (foreground), `angles-runbg` (background w/ PID), `angles-kill`
- **Git**: `angles-gitinit/gitcommit/gitlog/gitdiff/gitbranch`
- **Web**: `angles-fetch`, `angles-websearch`
- **Dir / mgmt**: `angles-ls/tree/cd/pwd`, `angles-fileinfo`, `angles-*dir/copy`

**Safety model**: reads require no approval; writes follow the configured approval policy; deletions and dangerous operations always require user confirmation (see [AGENTS.md](AGENTS.md)).

### 2. Multi-provider, one wire protocol

Provider registry (`provider.rs` + `providers.toml`) maps each of the 11 providers to its native wire protocol — OpenAI Chat Completions, Anthropic Messages, or Gemini Native — and normalizes the streaming + tool-call path so the rest of the codebase stays provider-agnostic.

### 3. AI operation plans (v0.4.1+)

Before touching anything, Angles renders a high-level plan from the system prompt and reports it in the terminal: *read… install… create… edit… run…*. Every step is visible and auditable; the agent does not operate silently.

### 4. Local HTTP gateway (v0.2+)

`angles serve` starts an embedded axum server on `http://127.0.0.1:8080` — a web console to edit config, switch providers, and test conversations in the browser. REST surface: `/health`, `/api/config`, `/api/providers`, `/api/chat`.

### 5. Privacy-first config

Everything lives in `~/.angles/config.json`. API keys can be read from a local file or set via the `ANGLES_API_KEY` environment variable — never transmitted to a relay.

---

## Configuration

Stored at `~/.angles/config.json`:

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

API key may also be supplied through the `ANGLES_API_KEY` environment variable.

---

## Installation

All four methods install the *same* binary:

<table>
<tr>
<td>

**npm (recommended)**

```bash
npm i -g @angleschina/angles && angles install
```

Routes through the npmmirror registry by default for the fastest fetch in CN.

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

When a prebuilt binary is available the installer **skips the Rust / compiler toolchain entirely** and downloads a ~1.6 MB binary in seconds. Works on iSH, Raspberry Pi, Alpine, and rootless environments.

---

## Quick start

```bash
# First-time setup (5-step TUI wizard)
angles gateway

# Interactive session (default mode)
angles

# One-shot non-interactive execution
angles exec "write a Python HTTP server"

# Emit an AI operation plan without executing (v0.4.1+)
angles plan "add JWT auth to this Express project, with tests"

# Start the local HTTP gateway (v0.2+) — http://127.0.0.1:8080
angles serve

# Inspect config / diagnose installation / help
angles config
angles doctor
angles help
```

---

## Supported providers

| Provider | API Host | Protocol |
|---|---|---|
| OpenAI | api.openai.com | OpenAI Chat Completions |
| Claude (Anthropic) | api.anthropic.com | Anthropic Messages API |
| Gemini (Google) | generativelanguage.googleapis.com | Gemini Native API |
| DeepSeek | api.deepseek.com | OpenAI Chat Completions |
| Grok (xAI) | api.x.ai | OpenAI Chat Completions |
| MiniMax | api.minimax.chat | OpenAI Chat Completions |
| OpenRouter | openrouter.ai | OpenAI Chat Completions |
| Qwen (通义千问) | dashscope.aliyuncs.com | OpenAI Chat Completions |
| GLM (智谱) | api.siliconflow.cn | OpenAI Chat Completions |
| Kimi (Moonshot) | api.moonshot.cn | OpenAI Chat Completions |
| Custom | user-defined | OpenAI / Anthropic / Gemini |

---

## Build from source

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone & build
git clone https://github.com/ZSJ305/angles-cli.git
cd angles-cli
cargo build --release

# Install
cp target/release/angles ~/.local/bin/
```

### Cross-compile

```bash
make setup-arm64 && make arm64   # Linux ARM64
make setup-x64 && make x64       # Linux x64
make macos-arm64                  # macOS ARM64 (macOS only)
```

Prebuilt binaries for all 5 platforms are produced automatically by GitHub Actions on every tag push — see [Releases](https://github.com/ZSJ305/angles-cli/releases).

---

## License

**MIT** — see [LICENSE](LICENSE).
