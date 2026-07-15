# hirn

<p align="center">
  <strong>An assistant you actually own.</strong>
</p>

<p align="center">
  Modular. Private. Distributed. Your data stays on your hardware. No cloud bills, no telemetry, no exceptions.
</p>

---

## 🧠 What is hirn?

Hirn is a composable agent system that you own and control. By default, every app, model execution, and learning loop runs 100% locally on your machine. We treat the file system as the source of truth, enabling you to use external tools (Vim, Git, Grep) alongside your AI agents.

## 🏛️ The System Architecture

Hirn is built on a **Local-First** foundation, ensuring that you are never locked into a proprietary cloud.

### The Storage Hierarchy

We maintain a three-tier storage model that balances human usability with machine queryability:

1.  **Tier 1 (Canonical)**: User data is stored as **human-readable files** (Markdown, JSON). This ensures 100% data ownership and longevity.
2.  **Tier 2 (Collaboration)**: Binary CRDT overlays handle real-time sync across devices, ensuring conflict-free collaboration.
3.  **Tier 3 (Queryable Indices)**: A multi-modal indexing layer containing **SQLite** (relational data), **Grafeo** (graph knowledge), caching, and **Vector DBs** (RAG/semantic search).

## 🛠️ The 6 Pillars

- **`router`**: Intelligent local-first gateway. Routes prompts to the optimal model based on task difficulty and hardware capability.
- **`agent`**: Rust-based orchestration engine. Handles complex ACP (Agent Control Protocol) tasks and workflow automation.
- **`learn`**: Local feedback loop. A fine-tuning classifier that optimizes routing accuracy based on your usage patterns.
- **`server`**: RPC-based model host. Orchestrates `llama.cpp` / vLLM backends, featuring VRAM sharding and load balancing.
- **`assistant`**: Extensible application ecosystem built in Flutter and Tauri. Build your own modular HTML/JS tools that run offline via a secure, bidirectional Typscript SDK.
- **`transcribe`**: Private voice input. Local-first transcription using Whisper, piping text directly to the router.

## 🚀 For Developers: Ship nothing to the cloud

Develop a tool, plug it into the Hirn ecosystem, and forget about the infrastructure.

- **Zero Infrastructure**: Tools run on the user's device. No backend to operate, no database to secure.
- **Rich SDK**: Your tools access the **Data Layer** (Tier 1-3) directly via TypeScript, gaining native support for persistence, RAG, collaboration, offline work and encrypted P2P syncronization across network boundaries.
- **FFI-backed Performance**: Logic runs in a high-performance **Rust Sync Core** shared across Flutter (mobile), Tauri (desktop) and the agent CLI.

## 📂 Module Map

| Module        | Description                                                         |
| :------------ | :------------------------------------------------------------------ |
| `agent/`      | ACP compliant orchestration engine and wrapper around pi.           |
| `assistant/`  | Mobile client (Flutter + Rust) for on-the-go access.                |
| `data/`       | Persistence management (File/SQLite/Vector/Graph).                  |
| `desktop/`    | Tauri-based cross-platform GUI & tool host.                         |
| `homepage/`   | Homepage & documentation portal.                                    |
| `router/`     | Intent classification & model dispatching.                          |
| `sdk/`        | TypeScript SDK for building modular tools using the hirn ecosystem. |
| `server/`     | Distributed inference orchestration.                                |
| `transcribe/` | Local privacy-first speech-to-text.                                 |

---

## 📜 License

Hirn is built for sovereignty. You are free to use and modify the code for non-commerical use. See [LICENSE](LICENSE) for details.
