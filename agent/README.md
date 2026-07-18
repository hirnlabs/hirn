# Agent

This directory contains the Rust wrapper implementation for the [Agent Client Protocol (ACP)](https://modelcontextprotocol.io/). It facilitates seamless communication between Hirn and any ACP-compliant agent. A custom implementation of `pi` is bundled within this module to handle agent-specific tasks and execution.

## Key Capabilities

- **ACP Integration**: Provides native Rust bindings for the Agent Client Protocol.
- **Custom Goose Bundle**: Includes a specialized `goose` agent implementation for internal task execution.
- **Orchestration**: Logic to chain multiple agent steps together.
- **Plugin System**: Interfaces for extending agent capabilities with custom tools.
- **Monitoring & Data collection**: Telemetry and logs for agent execution paths and finetuning using hirn learn.
