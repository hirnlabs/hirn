# Context Map

## Contexts

- [Agent](./agent/CONTEXT.md) — ACP compliant orchestration engine and workflow automation.
- [Assistant](./assistant/CONTEXT.md) — Mobile client (Flutter + Rust) for on-the-go access.
- [Data](./data/CONTEXT.md) — Persistence management (File/SQLite/Vector/Graph).
- [Desktop](./desktop/CONTEXT.md) — Tauri-based cross-platform GUI & tool host.
- [Homepage](./homepage/CONTEXT.md) — Web dashboard & documentation portal.
- [Router](./router/CONTEXT.md) — Intent classification & model dispatch logic.
- [SDK](./sdk/CONTEXT.md) — TypeScript SDK for building modular tools.
- [Server](./server/CONTEXT.md) — Distributed inference orchestration.
- [Transcribe](./transcribe/CONTEXT.md) — Local privacy-first speech-to-text.

## Relationships

- **Agent → Data**: Agent reads and writes data through the Tier 1-3 Storage Hierarchy.
- **Assistant → Server**: Assistant connects to local/remote inference server via RPC.
- **Desktop → Agent / Router**: Desktop GUI launches Agent workflows and queries Router for model dispatch.
- **Router → Server**: Router dispatches prompts to optimal models hosted by the Server.
- **SDK → Data**: SDK provides tools access to the Data Layer (Tier 1-3).
- **Transcribe → Router**: Transcribe pipes transcriptions directly to the Router.
- **Assistant / Desktop ↔ SDK**: Shared frontend-backend SDK logic.
