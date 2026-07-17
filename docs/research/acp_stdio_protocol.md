# ACP / MCP Stdio Protocol & Connection Initialization

The Model Context Protocol (MCP) and Agent Client Protocol (ACP) utilize **JSON-RPC 2.0** for all communication. When the `stdio` transport is used, messages are transmitted via standard input (`stdin`) and standard output (`stdout`), with standard error (`stderr`) strictly reserved for logging.

## 1. Message Framing over Stdio Pipes

Unlike the Language Server Protocol (LSP) which relies on HTTP-style `Content-Length` headers, MCP over stdio uses a simpler **newline-delimited JSON (NDJSON)** framing mechanism:

*   **One Message per Line:** Every JSON-RPC message (request, response, or notification) must be serialized as a single, valid JSON object on exactly one line.
*   **Newline Delimiter:** Messages are separated by standard newline characters (`\n`).
*   **No Embedded Newlines:** The JSON payload itself must **not** contain unescaped newline characters. Any newlines within string payloads must be escaped (e.g., `\n`).
*   **UTF-8 Encoding:** All JSON data must be encoded in UTF-8.
*   **Exclusive Stdio Usage:** 
    *   **Client to Server:** Written to the server process's `stdin`.
    *   **Server to Client:** Written to the server process's `stdout`.
    *   **Logging:** The server **must not** emit anything to `stdout` other than valid JSON-RPC messages. All debug logs, errors, or unstructured text must be sent to `stderr`.

## 2. JSON-RPC 2.0 Schema & Handshake

The protocol relies on a mandatory, two-step initialization handshake immediately after process startup. No other requests (except `ping` or logging) can be processed until this handshake completes.

### A. The `initialize` Request (Client → Server)
The client begins the handshake by sending an `initialize` request to negotiate protocol versioning and capabilities.

**Schema:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "protocolVersion": "2024-11-05",
    "capabilities": {
      "roots": { "listChanged": true },
      "sampling": {}
    },
    "clientInfo": {
      "name": "hirn-client",
      "version": "1.0.0"
    }
  }
}
```

### B. The `initialize` Response (Server → Client)
The server responds with its accepted protocol version and the features it supports (like `tools`, `resources`, `prompts`).

**Schema:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "protocolVersion": "2024-11-05",
    "capabilities": {
      "tools": { "listChanged": true },
      "resources": {},
      "prompts": {}
    },
    "serverInfo": {
      "name": "hirn-mcp-server",
      "version": "1.0.0"
    }
  }
}
```

### C. The `initialized` Notification (Client → Server)
Once the client receives and processes the server's `initialize` response, it must send an `initialized` notification. As a JSON-RPC notification, it does not include an `id` field.

**Schema:**
```json
{
  "jsonrpc": "2.0",
  "method": "notifications/initialized",
  "params": {}
}
```

### D. General Message Packets
After initialization, general communication adheres strictly to standard JSON-RPC 2.0 structures:

**General Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "example_tool",
    "arguments": { "arg1": "value" }
  }
}
```

**General Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "Tool executed successfully."
      }
    ]
  }
}
```
