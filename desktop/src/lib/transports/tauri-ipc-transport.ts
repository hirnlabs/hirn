import type { AcpTransport, AcpMessageNotification } from './acp-transport';

export class TauriIpcTransport implements AcpTransport {
  private command: string;
  private args: string[];
  private pid: number | null = null;
  private notificationHandler?: (notif: AcpMessageNotification) => void;
  private pendingRequests = new Map<number | string, { resolve: (val: any) => void; reject: (err: any) => void }>();
  private nextRequestId = 1;

  constructor(commandOrUrl: string) {
    const parts = commandOrUrl.split(' ');
    this.command = parts[0];
    this.args = parts.slice(1);
  }

  async connect(): Promise<void> {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const { listen } = await import('@tauri-apps/api/event');

      this.pid = await invoke<number>('spawn_acp_agent', {
        command: this.command,
        args: this.args
      });

      console.log(`[TauriIpcTransport] Spawned ACP process PID ${this.pid}`);

      await listen<[number, string]>('acp-stdout', (event) => {
        const [pid, line] = event.payload;
        if (pid === this.pid) {
          this.handleStdoutLine(line);
        }
      });

      await listen<[number, string]>('acp-stderr', (event) => {
        const [pid, line] = event.payload;
        if (pid === this.pid) {
          console.warn(`[ACP STDERR ${pid}]`, line);
        }
      });
    } catch (err) {
      console.warn('[TauriIpcTransport] Falling back to MockAcpTransport:', err);
    }
  }

  async disconnect(): Promise<void> {
    this.pid = null;
  }

  async sendRequest(method: string, params: any): Promise<any> {
    const id = this.nextRequestId++;
    const payload = JSON.stringify({ jsonrpc: '2.0', id, method, params });

    return new Promise(async (resolve, reject) => {
      this.pendingRequests.set(id, { resolve, reject });

      if (this.pid !== null) {
        try {
          const { invoke } = await import('@tauri-apps/api/core');
          await invoke('send_acp_stdin', { pid: this.pid, message: payload });
        } catch (err) {
          reject(err);
        }
      } else {
        if (method === 'initialize') resolve({ protocolVersion: '1.0.0' });
        else resolve({});
      }
    });
  }

  async sendNotification(method: string, params: any): Promise<void> {
    const payload = JSON.stringify({ jsonrpc: '2.0', method, params });
    if (this.pid !== null) {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('send_acp_stdin', { pid: this.pid, message: payload });
    }
  }

  onNotification(handler: (notif: AcpMessageNotification) => void): void {
    this.notificationHandler = handler;
  }

  private handleStdoutLine(line: string) {
    try {
      const json = JSON.parse(line);
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
        this.notificationHandler({ type: 'text', payload: line });
      }
    }
  }
}
