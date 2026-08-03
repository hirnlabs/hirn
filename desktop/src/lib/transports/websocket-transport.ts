import type { AcpTransport, AcpMessageNotification } from './acp-transport';

export class WebSocketAcpTransport implements AcpTransport {
  private url: string;
  private ws: WebSocket | null = null;
  private notificationHandler?: (notif: AcpMessageNotification) => void;
  private pendingRequests = new Map<number | string, { resolve: (val: any) => void; reject: (err: any) => void }>();
  private nextRequestId = 1;

  constructor(url: string) {
    this.url = url;
  }

  async connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      try {
        this.ws = new WebSocket(this.url);
        this.ws.onopen = () => {
          console.log(`[WebSocketAcpTransport] Connected to ${this.url}`);
          resolve();
        };
        this.ws.onerror = (err) => {
          console.warn(`[WebSocketAcpTransport] WebSocket error:`, err);
          reject(err);
        };
        this.ws.onmessage = (event) => {
          this.handleMessage(event.data);
        };
      } catch (err) {
        reject(err);
      }
    });
  }

  async disconnect(): Promise<void> {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }

  async sendRequest(method: string, params: any): Promise<any> {
    const id = this.nextRequestId++;
    const payload = JSON.stringify({ jsonrpc: '2.0', id, method, params });

    return new Promise((resolve, reject) => {
      this.pendingRequests.set(id, { resolve, reject });
      if (this.ws && this.ws.readyState === WebSocket.OPEN) {
        this.ws.send(payload);
      } else {
        reject(new Error('WebSocket connection is not open'));
      }
    });
  }

  async sendNotification(method: string, params: any): Promise<void> {
    const payload = JSON.stringify({ jsonrpc: '2.0', method, params });
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(payload);
    }
  }

  onNotification(handler: (notif: AcpMessageNotification) => void): void {
    this.notificationHandler = handler;
  }

  private handleMessage(data: string) {
    try {
      const json = JSON.parse(data);
      if (json.id !== undefined && this.pendingRequests.has(json.id)) {
        const { resolve } = this.pendingRequests.get(json.id)!;
        this.pendingRequests.delete(json.id);
        resolve(json.result || json);
      } else if (json.method && this.notificationHandler) {
        this.notificationHandler({
          type: 'text',
          payload: JSON.stringify(json.params || json)
        });
      }
    } catch {
      if (this.notificationHandler) {
        this.notificationHandler({ type: 'text', payload: data });
      }
    }
  }
}
