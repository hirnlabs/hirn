# Proxy Context

The tool-call control plane of Hirn: agents describe tool calls to the proxy, the proxy resolves, summarizes, and gates them — the caller executes.

## Language

**Proxy**:
The standalone CLI (`hirn-proxy`, exposed as `hirn run|list|add|activate|deactivate|skill`) that resolves a requested tool call into a normalized summary plus a policy verdict. It never executes tools on behalf of an agent.
_Avoid_: Tool runner, sandbox, ACP server

**Tool Definition**:
A directory under `~/.hirn/tools/<name>/` (global) or `.agents/tools/<name>/` (local) declaring a tool as an argv template plus a typed input schema and an optional summary template — never a shell string. Definitions are the single source of truth for what a tool is.

**Tool Index (`tools.json`)**:
The machine-generated file (`~/.hirn/tools.json` globally, `.agents/tools.json` locally) mirroring every definition in MCP `tools/list` format. Only two fields per entry are user-owned and survive re-sync: `active` and `policy`.
_Avoid_: Treating `tools.json` as a place to define tools

**Policy**:
The per-tool execution gate: `off` (hidden from discovery, invocation refused), `ask` (summarized, confirmation required), `full` (auto-approved).

**Caller Detection**:
TTY-based output switching: non-TTY callers (agents) receive standard tool-call JSON; terminal callers get pretty output and interactive prompts. Overridable with `--json` / `--interactive`.

**Resolution**:
Looking up a name across local and global directories, where a same-named local entry overrides its global one. Every call result exposes the resolved definition, its source path, the exact substituted argv, raw arguments, policy, and summary.

**Delegation**:
The existing `hirn` agent binary forwarding the proxy subcommands to the standalone `hirn-proxy` binary, so the public API is `hirn <verb>` from day one and later integration is a pure implementation swap.
