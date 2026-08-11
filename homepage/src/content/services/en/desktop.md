---
title: "desktop"
description: "Tauri v2 & Svelte 5 multi-agent desktop GUI and tool host for Windows, macOS, and Linux."
lead: "A high-density, minimal desktop interface. Built with Rust using Tauri v2, Svelte 5, and TypeScript for multi-agent ACP chat, sandboxed MCP App hosting, non-blocking execution, and seamless CLI sync."
icon: "desktop"
highlights:
  - title: "Multi-Agent ACP Chat"
    description: "Connect to multiple local (hirn acp, goose, claude) and remote ACP agents in a single unified interface."
  - title: "Tauri v2 + Svelte 5"
    description: "Ultra-fast native desktop shell with reactive Svelte 5 Runes ($state, $derived) for seamless session management."
  - title: "Canonical CLI Sync"
    description: "Shared session storage at ~/.hirn/sessions/ keeps chat history 100% in sync between CLI and Desktop."
  - title: "Auto-Update"
    description: "Native background startup updates via tauri-plugin-updater with single-click seamless restarts."
---

![Hirn Desktop Interface](/assets/desktop-dark.png)

## Install

Hirn supports macOS, Linux, and Windows. You can run the full-featured Desktop GUI or the standalone Agent CLI (or both).

### Desktop Application (GUI)

Download pre-built desktop installers for your operating system directly from [GitHub Releases](https://github.com/hirnlabs/hirn/releases/latest):

- **macOS:** Download `.dmg` disk image.
- **Windows:** Download `.exe` or `.msi` installer.
- **Linux:** Download `.AppImage` or `.deb` package.

> **Automatic Updates:** Desktop installations automatically check for updates on startup, download release updates in the background, and prompt to restart.

---

### Agent CLI (Headless / Standalone)

The Hirn Agent is a Rust-based ACP orchestration engine and CLI tool based on the goose agent by the Agentic AI Foundation.

```bash
# macOS / Linux / WSL2
curl -fsSL https://raw.githubusercontent.com/hirnlabs/hirn/main/agent/setup/install.sh | bash
```

```powershell
# Windows PowerShell
irm https://raw.githubusercontent.com/hirnlabs/hirn/main/agent/setup/install.ps1 | iex
```

---

### Standalone Web Client (Docker)

Deploy the Web Client as a lightweight Nginx/SvelteKit Docker container:

```bash
docker run -d \
  -p 80:80 \
  --name hirn-web \
  ghcr.io/hirnlabs/desktop-web:latest
```

---

## Quick start

### 1. Verify Setup & Configure Models
```bash
hirn doctor
hirn configure
```

### 2. Launch Terminal Sessions
```bash
# Interactive Chat Session
hirn session

# Terminal UI (TUI)
hirn tui
```

### 3. Connect Desktop & Web Bridge
Launch the **Hirn Desktop App** for multi-agent chat, non-blocking background execution, interactive tool UIs (`ext-apps`), and encrypted WebRTC P2P sync.

```bash
hirn desktop
```

To start an ACP server over WebSocket and HTTP for Web Client or remote tool connections:
```bash
hirn serve
```

---

## How it fits together

Hirn connects desktop apps, terminal interfaces, mobile clients, and local inference servers through an intelligent router and encrypted P2P signaling network with zero cloud lock-in:

```mermaid
flowchart TD
    subgraph Apps ["Applications"]
        Desktop["Desktop GUI (Tauri)"]
        Mobile["Mobile App (Flutter)"]
        SyncEngine["Common Protocol & Sync Engine (Rust)"]

        Desktop --> SyncEngine
        Mobile --> SyncEngine
    end

    subgraph MCP ["MCP Tooling"]
        Zeug["Zeug (Applets)"] -- "MCP Apps" --> MCPServer["MCPv2 Server"]
        Workflows["Workflows"] -- "MCP Tasks" --> MCPServer
    end

    SyncEngine <--"RPC"--> Cloud["Hirn Sync / message relay (Rust)"]

    SyncEngine -- "IPC / WS" --> Core["Agent Core & CLI (goose/Rust)"]
    MCPServer -- "STDIO / RPC" --> Core

    Cloud <--"RPC"--> Core

    subgraph Data ["Data Management"]
        T2["Tier 2: CRDT Sync"]
        T3["Tier 3: Indices / DB"]
        T1["Tier 1: Markdown Files"]

        T2 <--> T3
        T3 <--> T1
    end

    subgraph Inference ["Inference"]
        Router["Intelligent Model Router / Gateway (Rust)"]
        LLMs["Local Inference (llama.cpp/vLLM/colibri...)"]
        STT["Transcription / Speech-to-Text (Whisper)"]
        CloudLLMs["Cloud API Providers (optional)"]

        Router --> LLMs
        Router -. "optional" .-> CloudLLMs
    end

    Core <--> T3
    Core <--> T2
    Core <--> T1
    Core --> Router
    Core --> STT
```

- **Desktop Host:** Tauri v2 + SvelteKit multi-agent host with dual-mode transports (Tauri IPC, WebSocket, WebRTC P2P) and sandboxed interactive tool UIs (`ext-apps`).
- **Agent CLI:** Rust-based ACP orchestration engine, terminal UI (`tui`), and WebRTC ACP relay server.
- **Router & Server:** Intelligent local-first gateway routing prompts based on task complexity and VRAM/hardware capability across local llama.cpp / vLLM backends, local Whisper transcription, and optional cloud model providers.
- **Signaling & Relay Server:** Minimal Rust WebRTC server providing encrypted P2P synchronization and store-and-forward message queuing for async offline delivery across devices.
- **Storage Hierarchy (Tier 1-3):** Canonical human-readable files (Markdown/JSON), binary CRDT collaboration overlays, and SQLite, Grafeo graph knowledge, and Vector DB queryable indices.
- **Assistant & Transcribe:** Mobile companion app (Flutter + Rust Sync Core) and local privacy-first speech-to-text engine using Whisper.

---

## Security & Privacy

- **100% Local-First:** Sessions and configurations are persisted locally in `~/.hirn` and your file system. No unexpected cloud telemetry.
- **Sandboxed Tool UIs:** Interactive extension UIs run in isolated, sandboxed views with zero open HTTP ports.
- **Encrypted Relay:** WebRTC P2P sync uses encrypted store-and-forward message queues for secure inter-device communication.
- **Configurable Endpoints:** Swap signaling and relay endpoints to self-hosted instances seamlessly across Desktop UI, Agent CLI, and Mobile Assistant.
