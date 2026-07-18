# Hirn Agent

This directory contains the Hirn Agent—a Rust-based orchestration engine and CLI wrapper implementing the [Agent Client Protocol (ACP)](https://modelcontextprotocol.io/). It facilitates seamless communication, command execution, and task coordination.

> [!IMPORTANT]
> **Not a New Agent**: `hirn` is not a custom agent written from scratch. It is an adapted version of the **goose** agent, which is an open-source agent maintained by the **Agentic AI Foundation (AAIF)** at the Linux Foundation.

---

## Installation

You can install the Hirn Agent using the setup scripts located in [agent/setup](file:///C:/dev/hirn/agent/setup/). These scripts automatically detect your operating system and architecture, download the correct release asset, create the self-contained directory structure under `~/.hirn` (`~/.hirn/bin` and `~/.hirn/config`), and copy the binary.

### Linux and macOS

To download and install the latest published release:
```bash
curl -fsSL https://raw.githubusercontent.com/hirnlabs/hirn/main/agent/setup/install.sh | bash
```

To install a specific version/tag:
```bash
curl -fsSL https://raw.githubusercontent.com/hirnlabs/hirn/main/agent/setup/install.sh | bash -s -- --version v0.1.0
```

### Windows (PowerShell)

To download and install the latest published release:
```powershell
irm https://raw.githubusercontent.com/hirnlabs/hirn/main/agent/setup/install.ps1 | iex
```

To install a specific version/tag:
```powershell
$script = [scriptblock]::Create((irm https://raw.githubusercontent.com/hirnlabs/hirn/main/agent/setup/install.ps1))
& $script -Version v0.1.0
```

---

## Key Capabilities

- **ACP Integration**: Native Rust bindings and client/server protocol implementation.
- **Bundled Agent**: Integrates the Goose CLI directly within the binary for internal task execution.
- **Relay Server**: Includes a WebRTC ACP relay server for remote/distributed agent execution.
- **Self-Contained Configs**: Keeps all agent configurations, states, version/help caches, and temporary files within `~/.hirn`.

---

## CLI Usage

Run `hirn --help` to view all available commands. The CLI automatically delegates appropriate commands to the internal Goose implementation:

```
Hirn Agent CLI and WebRTC ACP Relay Server

Usage: hirn [COMMAND] [OPTIONS]

Hirn Agent Commands:
  relay                          Start the WebRTC ACP relay server
                                 (Run 'hirn relay --help' for options)

Hirn CLI Options & Usage (forwarded to Goose):
--------------------------------------------------------------------------------
An AI agent

Usage: hirn [COMMAND]

Commands:
  configure     Configure hirn settings
  info          Display hirn information
  doctor        Check that your Hirn setup is working
  mcp           Run one of the mcp servers bundled with hirn
  acp           Run hirn as an ACP agent server on stdio
  serve         Start ACP server over HTTP and WebSocket
  session       Start or resume interactive chat sessions [aliases: s]
  project       Open the last project directory [aliases: p]
  projects      List recent project directories [aliases: ps]
  run           Execute commands from an instruction file or stdin
  recipe        Recipe utilities for validation and deeplinking
  skills        Skill utilities
  plugin        Manage plugins
  schedule      Manage scheduled jobs [aliases: sched]
  gateway       Manage gateways for external platform integrations [aliases: gw]
  update        Update the hirn CLI version
  term          Terminal-integrated hirn session
  tui           Launch the hirn terminal UI
  local-models  Manage local inference models [aliases: lm]
  completion    Generate the autocompletion script or Nushell module for the specified shell
  review        Review the current diff using hirn
  help          Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```
