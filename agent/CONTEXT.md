# Agent Context

The ACP (Agent Control Protocol) compliant orchestration engine and workflow automation module for Hirn.

## Language

**Agent**:
The Rust-based orchestrator implementing the ACP and discovering local MCP App bundle tools.
_Avoid_: Custom AI model, chat backend

**ACP (Agent Control Protocol)**:
The protocol defining how agents communicate, execute commands, and coordinate tasks.

**Dual-Mode Execution**:
The agent capability routing tool invocations to a live app UI iframe over postMessage if open in Desktop/Assistant, or executing headlessly via `deno_core` / file / CRDT updates on the Rust agent side if closed.
_Avoid_: Headless browser spawning, UI background rendering

