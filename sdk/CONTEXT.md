# SDK Context

The TypeScript SDK (`@hirn/sdk`) for building modular, offline-first MCP Apps that run within Hirn hosts (Desktop, Assistant, Web, Standalone PWA).

## Language

**SDK**:
The TypeScript library allowing custom offline MCP App bundles to communicate with the Rust Sync Core and host applications via the postMessage bridge channel.
_Avoid_: Hirn FFI library, custom Zeug API

**MCP App (`ext-apps`)**:
An interactive web application (HTML/JS/CSS) served from an App Bundle, rendered in a sandboxed iframe, conforming to the Model Context Protocol user-interaction extension.
_Avoid_: Widget, custom tool, plugin, Zeug

**App Bundle**:
A self-contained directory under `apps/<app-id>/` containing a `manifest.json`, static web assets in `ui/`, and optional MCP tool definitions in `tools/`.
_Avoid_: Package, archive, plugin folder

**Hirn Host Protocol**:
The postMessage JSON-RPC contract and capability-upgrade mechanism between an MCP App iframe and the host application (Tauri Desktop or Flutter Assistant).
_Avoid_: FFI bridge, WebSocket protocol, custom IPC

**Capability Grant**:
An explicit, user-approved permission declared in `manifest.json` granting an app access to shared user files (`user-files/*`), system APIs, or network domains outside its private `apps/<app-id>/` namespace.
_Avoid_: Permission flag, system access bit

**PWA Shell**:
A 20-line HTML wrapper (`shell.html`) embedding an MCP App iframe when running standalone as a PWA, ensuring 100% `postMessage` protocol uniformity everywhere.
_Avoid_: PWA bridge script, Wasm container


