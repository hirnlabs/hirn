---
title: "agent"
description: "The desktop UI and CLI agent built on top of Goose."
lead: "The CLI agent and orchestration core you actually operate behind the scenes. hirn_agent is built directly on top of Goose and connects to tools via MCP v2, transforming it into a powerful local agent that can do stuff for you."
icon: "agent"
highlights:
  - title: "MCP v2 Ecosystem"
    description: "Fully supports the stateless MCP v2 core, MCP Apps spec for rich interfaces, and MCP Tasks for long-running workflows."
  - title: "Native Feel"
    description: "No clunky web wrappers. The agent feels like a native part of your operating system, deeply integrated with your existing developer tools."
  - title: "Background Execution"
    description: "The agent runs silently in your system tray, letting you push long-running tasks to the background and notifying you when they finish."
---

## Install in one command

Install scripts detect your OS and architecture, download the right release artifact, and place everything in `~/.hirn`.

**Windows (PowerShell)**

```powershell
irm https://raw.githubusercontent.com/hirnlabs/hirn/main/agent/setup/install.ps1 | iex
```

**Linux / macOS**

```bash
curl -fsSL https://raw.githubusercontent.com/hirnlabs/hirn/main/agent/setup/install.sh | bash
```

## Seamless ModelRegistry Integration

By leveraging Goose's extensible `ModelRegistry`, the **hirn_router** extension sits transparently in the background. It dynamically swaps the active model under the hood on a per-turn basis, ensuring that you maintain a completely unified chat history and context window without interruptions.

## Curated Desktop Plugins & MCP v2 Infrastructure

The agent comes packed with specialized local plugins and next-gen protocol standards that give Goose total mastery over your environment:

- **MCP v2 (Stateless Core):** Fully compliant with the 2026-07-28 Model Context Protocol revision. Eliminates protocol-level session state, enabling ultra-fast, stateless HTTP gateway routing and maximum scalability.
- **MCP Apps Specification:** All Zeug built inside the hirn ecosystem is fully MCP v2 compliant and leverages the official MCP Apps spec to deliver rich, server-rendered interactive interfaces directly to users.
- **MCP Tasks Extension:** Native support for the MCP Tasks specification, allowing long-running background jobs, asynchronous agent workflows, and reactive task notifications.
- **Agent Control Protocol (ACP):** Standardized interface for the agent to control your desktop UI and interact with applications.
- **Local File Access:** Secure, permission-scoped read/write capabilities for large local codebases.
- **Shell Execution:** Sandboxed terminal commands directly executed by the agent to run tests, build scripts, or manage git.
