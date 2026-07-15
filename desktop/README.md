# Desktop

The desktop client for Hirn, providing a cross-platform GUI for interacting with the Hirn services.

## Technology Stack

- **Framework**: Built using [Tauri](https://tauri.app/) for small footprint and performance.
- **Frontend**: Integrated with the common web components used in the homepage.
- **Backend Bridge**: Secure communication layer between the GUI and the underlying agent and server components.

## Tooling Ecosystem

- **Extensible Tools**: Support for modular "tools"-small HTML applications that can be added to the desktop.
- **Hirn SDK**: Tools interface directly with the Hirn SDK to access system resources (file system, hardware controls) and perform LLM inference via the local router/server. These tools leverage the **Data Layer** for persistent storage, state management, and semantic retrieval.
