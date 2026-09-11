# Proxy

The minimal tool-call control plane of Hirn. When an agent wants to call a tool, it does not execute anything itself — it describes the call to the proxy (`hirn run <tool> '<json-args>'`). The proxy resolves the tool definition, renders a complete summary of exactly what would run, and returns a policy verdict. The caller executes. New tools and skills can be added, activated, and permission-gated without touching any agent runtime.

> **Status**: Architectural decision set — v1 is specified but not yet implemented.

## Key Decisions

- **Control plane, not executor**: For agents, the proxy only translates a requested call into a normalized summary + policy verdict; the agent executes. In direct shell use it may execute local CLI tools itself, gated by an interactive `y/N`.
- **Plain CLI, no protocol**: Any agent that can run a shell command can use it. ACP is deferred entirely from v1; MCP (definitions included) and agent definitions (`~/.hirn/agents`) are out of scope for v1.
- **Policy per tool**: `off` (hidden and refused), `ask` (summarized, confirmation required), `full` (auto-approved). In shell mode: `full` runs immediately, `ask` shows the resolved command and prompts, `off` refuses.
- **Definitions are truth**: Tool definitions live in `tools/<name>/` directories as an argv template + typed input schema + optional summary template — never shell strings. `tools.json` is regenerated from them; only `active` and `policy` are user-owned.
- **Local overrides global**: A same-named entry in `.agents/` beats `~/.hirn/`; every result shows which definition resolved and from where.
- **Full transparency**: Every call result carries the resolved definition, source path, exact substituted argv, raw arguments, policy, and summary (declared template, or derived from the argv when absent).
- **TTY caller detection**: Non-TTY → standard tool-call JSON for agents; terminal → pretty output + interactive prompts. `--json` / `--interactive` override.
- **Standalone + delegated**: Built as its own `hirn-proxy` Rust binary for testing; the existing `hirn` agent binary delegates the subcommands, so the public API is `hirn <verb>` from day one.

## Commands

| Command | Purpose |
| --- | --- |
| `hirn list` | Merged view of active tools and skills (local over global, `off` filtered out). JSON in agent mode, pretty table in a terminal. Injected into agent context at session start. |
| `hirn run <tool> '<json-args>'` | Resolve, summarize, and gate a tool call. Agent mode returns the summary + verdict; shell mode executes per policy. |
| `hirn skill <name>` | Print the full SKILL.md of an active skill. |
| `hirn add <source>` | Fetch a directory or file from a GitHub-like source, detect skill vs tool by content, install it **inactive**. |
| `hirn activate <name>` / `hirn deactivate <name>` | Flip the `active` flag in `tools.json`; activation gates both listing and loading. |

## Configuration Layout

```
~/.hirn/
├── tools.json          # generated index (MCP tools/list format) + user-owned active/policy
└── tools/<name>/       # tool definitions — single source of truth

<project>/.agents/
├── tools.json          # local index, same rules
├── tools/<name>/       # local definitions, override global by name
└── skills/<name>/      # skills (existing layout)
```

## Agent Bootstrap

An `AGENTS.md` rules block instructs agents to run `hirn list` at session start and route every tool call through `hirn run`.

## Out of Scope (v1)

ACP, MCP entirely (definitions included), agent definitions in AFM format, skill groups, versioning/lockfiles, and an `update` command.
