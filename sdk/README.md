# SDK

The TypeScript SDK (`@hirn/sdk`) for building modular tools as standard **MCP Apps (`ext-apps`)**.

## Core Capabilities

- **MCP Apps Standard**: Wraps `@modelcontextprotocol/ext-apps` for sandboxed iframe execution.
- **Bridge Channel**: Upgrades `postMessage` JSON-RPC to carry Hirn CRDT, RPC, and file operations.
- **Data Scoping**: Scoped storage in `apps/<app-id>/` with capability grants for `user-files/*`.
- **Portable**: Runs in Tauri Desktop, Flutter Assistant, Web Dashboard, or standalone PWAs (`shell.html`).


