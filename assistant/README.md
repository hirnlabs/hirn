# Assistant

Cross-platform mobile application (Android/iOS) built with Flutter and Rust FFI.

## Technical Architecture

- **Frontend & Native Bridge**: Flutter UI with Rust FFI for high-performance logic and on-device model execution.
- **MCP Apps Host**: Renders MCP Apps (`ext-apps`) via native custom schemes (`WKURLSchemeHandler` / `WebViewAssetLoader`) under `hirn://` with 0 open HTTP ports.
- **Sync & Offloading**: Real-time CRDT state sync with RPC offloading to Hirn Server when needed.


