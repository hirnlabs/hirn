# Desktop Context

The Tauri and Svelte 5 based cross-platform GUI & multi-agent client host for Hirn.

## Domain Concepts & Language

**Desktop Host**:
The desktop client running Tauri v2 to package and execute offline-ready modular tools and multi-agent ACP orchestration workflows.
*Avoid*: Electron app, web view wrapper.

**Dual-Mode Transport (`AcpTransport`)**:
The unified transport abstraction interface allowing the Svelte 5 frontend to communicate with Agent Client Protocol (ACP) agents seamlessly across both Desktop (Tauri IPC) and Web Browser (WebSocket / P2P WebRTC) modes.
*Avoid*: Hardcoding Tauri `invoke()` or browser `WebSocket` calls inside UI components.

**Tauri IPC Transport (`TauriIpcTransport`)**:
The desktop transport driver executing local ACP agent binaries (`hirn acp`, `goose`, etc.) via Rust `tokio::process` child process pipes (`stdin`/`stdout`), emitting newline-delimited JSON (NDJSON) over Tauri IPC events.
*Avoid*: Browser `ChildProcess` polyfills.

**WebSocket Bridge Transport (`WebSocketTransport`)**:
The core transport driver connecting to `hirn serve --acp "<command>"` or network ACP WebSocket endpoints on `ws://` / `wss://`. Enables web browser mode and local network clients to interact with Hirn agents.
*Avoid*: Monolithic HTTP REST polling.

**P2P WebRTC Sync & Relay (`P2pWebRtcTransport`)**:
The multi-device synchronization transport enabling WebRTC P2P agent connections and encrypted store-and-forward message queuing (`https://agent.hirn-labs.com/connect?peer=<id>&relay=<url>&fingerprint=<sha256>`). The signaling relay server is open source and self-hostable, with configurable endpoints across Desktop UI, Agent CLI, and Mobile Assistant.
*Avoid*: Hardcoding relay URLs or making self-hosted relays incompatible.

**Commercial Licensing & Sync Model**:
The licensing framework requiring a License Key (one-time purchase) or active Hirn Sync Subscription for any commercial usage, while remaining free for personal/non-commercial use.
*Avoid*: DRM locks on local-first offline CLI features.

**Protocol Session Layer (`AcpClientSession`)**:
The protocol manager that wraps an `AcpTransport`, executes the mandatory ACP initialization handshake (`initialize` -> `result` -> `notifications/initialized`), negotiates capability flags, and exposes high-level agent methods (`sendPrompt`, `cancel`, `listTools`).
*Avoid*: Embedding ACP handshake state inside transport classes.

**Reactive Session Store (`SessionStore` & `AgentSession`)**:
The class-based reactive state model written in Svelte 5 Runes (`$state`, `$derived`, `$effect`) managing multiple concurrent agent sessions, active message threads, and transient tool execution states.
*Avoid*: Legacy Svelte 4 `writable()` stores, global Redux/Zustand stores.

**Canonical Session Persistence (`~/.hirn/sessions/`)**:
The disk storage directory holding session JSON files (`~/.hirn/sessions/<session_id>.json`), shared as the single source of truth across `hirn cli`, `hirn serve`, and `hirn desktop`.
*Avoid*: Isolated app-specific database schemas that fragment session history between CLI and GUI.

**Session Status Lifecycle (`AgentStatus`)**:
The real-time status state tracked per session:
- `idle` / `finished`: Task completed, waiting for next prompt.
- `working`: Actively thinking, streaming tokens, or executing tools (pulsing status).
- `waiting_for_input`: Agent requires user confirmation or tool permission approval (alert badge).
- `error`: Connection or execution failure.
*Avoid*: Modal dialog popups that block user navigation during session switching.

**Custom Scheme Protocol (`hirn://`)**:
The native URI scheme handler (`tauri::UriScheme`) serving local app bundles under `hirn://apps/<app-id>/index.html` with zero open HTTP ports, as well as handling deep links (`hirn://connect?...`).
*Avoid*: Localhost HTTP server, loopback socket.
