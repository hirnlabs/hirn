# MCP Apps (ext-apps) Specification Research

Research for [HIR-15](https://linear.app/hirnlabs/issue/HIR-15/research-mcp-apps-ext-apps-specification-deep-dive).

Primary sources:
- [ext-apps repository](https://github.com/modelcontextprotocol/ext-apps)
- [MCP Apps spec](https://modelcontextprotocol.io/specification/2025-06-18/user-interaction/apps)

## 1. postMessage JSON-RPC Protocol

Communication between the AI host and the embedded app happens entirely via the browser's native `window.postMessage` API, following the JSON-RPC 2.0 specification.

### Lifecycle Events

- **`ui/initialize`**: The host initiates the handshake to configure the app and pass initial context (theme, locale, capabilities).
- **`ui/notifications/host-context-changed`**: Sent by the host when context like the theme or locale changes.
- **`ui/notifications/tool-result`**: Used by the host to stream tool results to the UI.
- **`ui/notifications/tool-input`**: Used by the host to stream tool arguments to the UI.

### Theming

The host pushes theme updates (dark/light mode, CSS variables) to the app via `onhostcontextchanged` events. The app reacts to blend natively into the host's styling.

Source: [ext-apps README](https://github.com/modelcontextprotocol/ext-apps)

## 2. `App` Class API (`@modelcontextprotocol/ext-apps`)

### Constructor

```typescript
const app = new App(
  { name: "AppName", version: "1.0.0" },  // App info
  {}                                       // Host capabilities
);
```

### Methods

- **`connect()`**: Establishes the `PostMessageTransport` and performs the `ui/initialize` handshake.
- **`callServerTool(request)`**: Triggers tools on the connected MCP server from the UI.
- **`updateModelContext(context)`**: Sends context updates back to the host/model.

### Event Handlers (property-based)

- **`ontoolresult`**: Receives tool execution results.
- **`ontoolinput`** / **`ontoolinputpartial`**: Receives completed or streaming tool arguments.
- **`ontoolcancelled`**: Handles cancellation by the host/user.
- **`onhostcontextchanged`**: Reacts to theme or configuration changes.
- **`onteardown`**: Fired before the host unmounts the app.

Source: [ext-apps SDK source](https://github.com/modelcontextprotocol/ext-apps)

## 3. `@mcp-ui/client` Host-Side Package

- Provides components (`UIResourceRenderer`, `AppRenderer`) to safely render the sandboxed iframe.
- Manages the JSON-RPC bridge inside the parent window.
- Handles capability control (restricting whether the app can call certain tools or open links).

Source: [ext-apps repository](https://github.com/modelcontextprotocol/ext-apps)

## 4. Iframe Sandboxing & Security Model

- MCP Apps run in an **isolated iframe**.
- Standard sandbox attributes: `allow-scripts`, `allow-forms`, `allow-popups` (for link targets).
- Apps **cannot** access the parent window's DOM, cookies, or storage.
- A proxy or "double-iframe" architecture is sometimes used to enforce strict Content Security Policies (CSP) without `srcDoc` vulnerabilities.

Source: [MCP Apps specification](https://modelcontextprotocol.io/specification/2025-06-18/user-interaction/apps)

## 5. Declaring App UI in Tool Responses

- MCP servers declare that a tool has an associated UI by returning **`_meta.ui`** in the tool definition or tool result.
- **`_meta.ui.resourceUri`**: A URI (e.g., `ui://...`) pointing to the HTML/JS application to render.
- The host sees this metadata, fetches the resource, and spins up the iframe.

Source: [MCP Apps specification](https://modelcontextprotocol.io/specification/2025-06-18/user-interaction/apps)

## 6. Extension Points for Hirn Bridge Channel

- **Capability negotiation**: During `ui/initialize`, the app and host declare supported capabilities. This is the natural hook for Hirn to advertise bridge channel support.
- **Custom JSON-RPC methods**: The protocol's reliance on standardized JSON-RPC transport means hosts can extend the standard `ui/*` message namespace with custom schemas (e.g., `hirn/*`) if both sides support them.
- **No restriction on additional messages**: The postMessage channel is bidirectional and multiplexable — Hirn can interleave `hirn/*` bridge messages alongside standard `ui/*` messages on the same channel.

Source: [ext-apps SDK source](https://github.com/modelcontextprotocol/ext-apps), [MCP specification](https://modelcontextprotocol.io)
