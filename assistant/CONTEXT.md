# Assistant Context

The mobile client (Flutter + Rust) that provides on-the-go access to Hirn services and hosts modular MCP Apps.

## Language

**Assistant**:
The Flutter-based mobile application running on iOS and Android devices, integrated with Rust FFI for local processing.
_Avoid_: Mobile web app, wrapper app

**Mobile Custom Scheme**:
The platform-native custom protocol handler (`WKURLSchemeHandler` on iOS, `WebViewAssetLoader` on Android) serving local app bundles under `hirn://apps/<app-id>/index.html` with zero open ports.
_Avoid_: Embedded HTTP server, local web server

