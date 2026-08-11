import * as acp from '@agentclientprotocol/sdk';
import type { ExtendedAcpStream, AcpStreamOptions } from './tauri-ipc-transport';

export async function createWebSocketAcpStream(
  url: string,
  options?: AcpStreamOptions
): Promise<ExtendedAcpStream> {
  return new Promise((resolve, reject) => {
    try {
      const ws = new WebSocket(url);

      ws.onopen = () => {
        console.log(`[WebSocketAcpStream] Connected to ${url}`);

        const readable = new ReadableStream<acp.AnyMessage>({
          start(controller) {
            ws.onmessage = (event) => {
              if (typeof event.data === 'string' && event.data.trim()) {
                try {
                  const parsed = JSON.parse(event.data);
                  controller.enqueue(parsed);
                } catch {
                  console.warn(`[WebSocketAcpStream] Message parse warning:`, event.data);
                }
              }
            };
            ws.onerror = (err) => {
              console.warn(`[WebSocketAcpStream] Error:`, err);
              if (options?.onError) {
                options.onError({
                  source: 'connection',
                  message: `WebSocket error on ${url}`
                });
              }
            };
            ws.onclose = () => {
              try { controller.close(); } catch {}
            };
          },
          cancel() {
            ws.close();
          }
        });

        const writable = new WritableStream<acp.AnyMessage>({
          write(message) {
            if (ws.readyState === WebSocket.OPEN) {
              ws.send(JSON.stringify(message));
            } else {
              throw new Error('WebSocket connection is not open');
            }
          }
        });

        resolve({
          readable,
          writable,
          async disconnect() {
            ws.close();
          }
        });
      };

      ws.onerror = (err) => {
        reject(err);
      };
    } catch (err) {
      reject(err);
    }
  });
}
