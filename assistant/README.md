# Assistant

The mobile application for Hirn, available on both Android and iOS. Built with a [Flutter](https://flutter.dev/) frontend for a native cross-platform experience, utilizing [Rust](https://www.rust-lang.org/) bridges to handle high-performance logic and local inference.

## Technical Architecture

- **Frontend**: Flutter UI for intuitive chat, voice input, and system control.
- **Native Bridge**: Rust integration (via FFI) enabling heavy lifting and secure local processing on mobile hardware.
- **On-Device LLM**: Runs a local, tiny, tool-optimized model directly on the device for low-latency responses and offline capabilities.
- **Router Integration**: Incorporates the Hirn routing logic to classify user intent, determining whether tasks should be handled locally by the tiny model or offloaded.

## Connectivity & Offloading

- **Local-First**: Prioritizes on-device execution using the local model and router.
- **Remote Offloading**: Connects securely to a remote Hirn Server via RPC. When a task requires reasoning beyond the capacity of the local mobile model, the app transparently offloads the computation to your remote infrastructure.
- **Synchronization**: Seamlessly syncs conversation state and context windows between mobile, desktop, and server instances.

## Key Modules

- **Conversation State**: Manages memory and context windows, ensuring consistent interaction across devices.
- **Session Handler**: Orchestrates the lifecycle of user sessions and secure connections to remote hosts.
- **RPC Client**: Manages the communication protocol for tasks distributed to the Hirn Server.
- **Tooling Ecosystem**: Supports modular "tools"-small, self-contained HTML applications that access system resources and inference capabilities via the Hirn SDK, leveraging the **Data Layer** for persistence and retrieval.
