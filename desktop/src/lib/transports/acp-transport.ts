import type { AcpModel } from '../types/acp';

export interface AcpMessageNotification {
  type: 'thought' | 'text' | 'tool_call' | 'tool_result' | 'status' | 'models';
  payload: any;
}

export interface AcpTransport {
  connect(): Promise<void>;
  disconnect(): Promise<void>;
  sendRequest(method: string, params: any): Promise<any>;
  sendNotification(method: string, params: any): Promise<void>;
  onNotification(handler: (notif: AcpMessageNotification) => void): void;
}

export class MockAcpTransport implements AcpTransport {
  private notificationHandler?: (notif: AcpMessageNotification) => void;

  async connect(): Promise<void> {
    console.log('[MockAcpTransport] Connected to mock STDIO agent.');
  }

  async disconnect(): Promise<void> {
    console.log('[MockAcpTransport] Disconnected.');
  }

  async sendRequest(method: string, params: any): Promise<any> {
    if (method === 'initialize') {
      const models: AcpModel[] = [
        { id: 'local/gemma-4-9b-it', name: 'gemma-4-9b-it', provider: 'Local (GGUF)' },
        { id: 'local/qwen2.5-coder-7b', name: 'qwen2.5-coder-7b', provider: 'Local (GGUF)' },
        { id: 'anthropic/claude-3-5-sonnet', name: 'claude-3.5-sonnet', provider: 'Anthropic (ACP)' },
        { id: 'anthropic/claude-3-7-sonnet', name: 'claude-3.7-sonnet', provider: 'Anthropic (ACP)' },
        { id: 'lmstudio/llama3.2-3b-instruct', name: 'llama3.2-3b-instruct', provider: 'LMStudio' },
        { id: 'google/gemini-2.5-flash', name: 'gemini-2.5-flash', provider: 'Google (API)' }
      ];
      return {
        protocolVersion: '1.0.0',
        agentInfo: { name: 'Hirn Local Agent', version: '0.1.0' },
        capabilities: { tools: true, prompts: true },
        models
      };
    }
    if (method === 'session/prompt') {
      this.simulateStreamingResponse(params.prompt, params.modelId);
      return { status: 'ok' };
    }
    return {};
  }

  async sendNotification(method: string, params: any): Promise<void> {
    console.log('[MockAcpTransport] Notification sent:', method, params);
  }

  onNotification(handler: (notif: AcpMessageNotification) => void): void {
    this.notificationHandler = handler;
  }

  private simulateStreamingResponse(promptText: string, modelId?: string) {
    if (!this.notificationHandler) return;
    const handler = this.notificationHandler;
    const modelStr = modelId ? `[Model: ${modelId}] ` : '';

    setTimeout(() => {
      handler({ type: 'status', payload: 'working' });
    }, 100);

    setTimeout(() => {
      handler({ type: 'thought', payload: `${modelStr}Analyzing intent for "${promptText}". Inspecting local context.` });
    }, 400);

    setTimeout(() => {
      handler({
        type: 'tool_call',
        payload: {
          id: `tool-${Date.now()}`,
          name: 'read_workspace_file',
          arguments: { path: 'desktop/CONTEXT.md' },
          status: 'completed',
          result: 'Verified desktop context mapping.'
        }
      });
    }, 800);

    setTimeout(() => {
      handler({ type: 'text', payload: `${modelStr}Processed request for "${promptText}". Workspace context verified.` });
      handler({ type: 'status', payload: 'idle' });
    }, 1400);
  }
}
