import * as acp from '@agentclientprotocol/sdk';
import type { AcpModel, ThoughtLevelConfig, AcpModeConfig } from '../types/acp';
import type { ExtendedAcpStream } from './tauri-ipc-transport';

export type AcpNotificationCallback = (notif: {
  type: 'thought' | 'text' | 'tool_call' | 'tool_result' | 'status' | 'error';
  payload: any;
}) => void;

export interface AcpSessionResult {
  models: AcpModel[];
  thoughtLevelConfig: ThoughtLevelConfig | null;
  modeConfig: AcpModeConfig | null;
}

export class AcpClientSession {
  private stream: ExtendedAcpStream;
  private connection: acp.ClientConnection | null = null;
  private isInitialized = false;
  public sessionId: string | null = null;
  public fetchedModels: AcpModel[] = [];
  public thoughtLevelConfig: ThoughtLevelConfig | null = null;
  public modeConfig: AcpModeConfig | null = null;

  constructor(stream: ExtendedAcpStream) {
    this.stream = stream;
  }

  async startSession(onNotification: AcpNotificationCallback, cwd?: string): Promise<AcpSessionResult> {
    const clientApp = acp
      .client({ name: 'hirn-desktop' })
      .onNotification(acp.methods.client.session.update, (ctx) => {
        const update = ctx.params?.update as any;
        if (!update) return;

        const sUpdate = update.sessionUpdate || update.type;

        // Unpack text content from ACP update object/array variants
        const extractText = (obj: any): string | null => {
          if (!obj) return null;
          if (typeof obj === 'string') return obj;
          if (obj.text && typeof obj.text === 'string') return obj.text;
          if (obj.delta && typeof obj.delta === 'string') return obj.delta;
          if (obj.thought && typeof obj.thought === 'string') return obj.thought;
          if (obj.content) {
            if (typeof obj.content === 'string') return obj.content;
            if (obj.content.text) return obj.content.text;
            if (Array.isArray(obj.content)) {
              return obj.content
                .map((c: any) => (typeof c === 'string' ? c : c.text || c.delta || ''))
                .join('');
            }
          }
          return null;
        };

        if (sUpdate === 'agent_thought_chunk' || update.thought || sUpdate === 'thought') {
          const text = extractText(update);
          if (text) onNotification({ type: 'thought', payload: text });
        } else if (
          sUpdate === 'agent_message_chunk' ||
          sUpdate === 'message_chunk' ||
          sUpdate === 'text_chunk' ||
          update.text ||
          update.delta ||
          update.content
        ) {
          const text = extractText(update);
          if (text) onNotification({ type: 'text', payload: text });
        } else if (sUpdate === 'tool_call' || update.toolCall) {
          onNotification({ type: 'tool_call', payload: update.toolCall || update });
        } else if (update.error) {
          onNotification({ type: 'error', payload: update.error });
        }
      })
      .onRequest(acp.methods.client.session.requestPermission, async (ctx) => {
        const firstOption = ctx.params.options?.[0]?.optionId || 'allow';
        return {
          outcome: {
            outcome: 'selected',
            optionId: firstOption
          }
        };
      });

    this.connection = clientApp.connect(this.stream);

    const initResult = await this.connection.agent.request(acp.methods.agent.initialize, {
      clientInfo: { name: 'Hirn Desktop', version: '0.1.0' },
      protocolVersion: acp.PROTOCOL_VERSION
    });

    let models: AcpModel[] = [];

    if (initResult && Array.isArray((initResult as any).models) && (initResult as any).models.length > 0) {
      models = (initResult as any).models;
    }

    try {
      const workingDir = cwd || 'C:\\dev\\hirn';
      const newSessionRes = await this.connection.agent.request(acp.methods.agent.session.new, {
        cwd: workingDir,
        mcpServers: []
      });

      if (newSessionRes && newSessionRes.sessionId) {
        this.sessionId = newSessionRes.sessionId;
      }

      // 1. Check direct modes payload from NewSessionResponse
      if (newSessionRes && (newSessionRes as any).modes) {
        const modesRes = (newSessionRes as any).modes;
        if (modesRes.availableModes && Array.isArray(modesRes.availableModes)) {
          this.modeConfig = {
            currentModeId: modesRes.currentModeId || modesRes.availableModes[0]?.id || 'default',
            modes: modesRes.availableModes.map((m: any) => ({
              id: m.id || m.name,
              name: m.name || m.id,
              description: m.description
            }))
          };
        }
      }

      // 2. Parse configOptions if available
      if (newSessionRes && Array.isArray((newSessionRes as any).configOptions)) {
        const configOptions = (newSessionRes as any).configOptions;

        // Parse model config
        const modelConfig = configOptions.find((c: any) => c.category === 'model' || c.id === 'model' || c.id === 'modelId');
        if (modelConfig && modelConfig.type === 'select') {
          const opts = modelConfig.options || [];
          const flatOptions = opts.flatMap((o: any) => o.options ? o.options : [o]);
          for (const opt of flatOptions) {
            const mId = opt.value || opt.id;
            if (mId && !models.some(m => m.id === mId)) {
              models.push({
                id: mId,
                name: opt.name || mId,
                provider: (opt as any).group || 'ACP Agent',
                thinking: { supported: true }
              });
            }
          }
        }

        // Parse thought_level config
        const thoughtConfig = configOptions.find((c: any) => c.category === 'thought_level');
        if (thoughtConfig && thoughtConfig.type === 'select') {
          const opts = thoughtConfig.options || [];
          const flatOptions = opts.flatMap((o: any) => o.options ? o.options : [o]);
          this.thoughtLevelConfig = {
            configId: thoughtConfig.id,
            currentValue: thoughtConfig.currentValue || flatOptions[0]?.value || 'mid',
            levels: flatOptions.map((o: any) => ({ value: o.value, name: o.name || o.value }))
          };
          console.log('[AcpClientSession] Thought level config:', this.thoughtLevelConfig);
        }

        // Parse mode config from configOptions if not already found in direct modes payload
        if (!this.modeConfig) {
          const mConfig = configOptions.find((c: any) => c.category === 'mode');
          if (mConfig && mConfig.type === 'select') {
            const opts = mConfig.options || [];
            const flatOptions = opts.flatMap((o: any) => o.options ? o.options : [o]);
            this.modeConfig = {
              configId: mConfig.id,
              currentModeId: mConfig.currentValue || flatOptions[0]?.value || 'default',
              modes: flatOptions.map((o: any) => ({
                id: o.value || o.id,
                name: o.name || o.value,
                description: o.description
              }))
            };
            console.log('[AcpClientSession] Mode config:', this.modeConfig);
          }
        }
      }
    } catch (e) {
      console.log('[AcpClientSession] Note: session/new response skipped or optional:', e);
    }

    if (models.length === 0 && initResult && (initResult as any).agentInfo) {
      const agentInfo = (initResult as any).agentInfo;
      models.push({
        id: `${agentInfo.name || 'hirn'}-default`,
        name: `${agentInfo.title || agentInfo.name || 'Hirn Agent'} (v${agentInfo.version || '0.1'})`,
        provider: 'ACP Agent',
        thinking: { supported: false }
      });
    }

    this.fetchedModels = models;
    this.isInitialized = true;
    console.log('[AcpClientSession] Connected natively via SDK with models:', this.fetchedModels, 'SessionId:', this.sessionId);

    return {
      models: this.fetchedModels,
      thoughtLevelConfig: this.thoughtLevelConfig,
      modeConfig: this.modeConfig
    };
  }

  async setMode(modeId: string, configId?: string): Promise<void> {
    if (!this.connection || !this.sessionId) return;
    try {
      if (configId) {
        await this.connection.agent.request(acp.methods.agent.session.setConfigOption, {
          sessionId: this.sessionId,
          configId,
          value: modeId
        });
      } else {
        await this.connection.agent.request(acp.methods.agent.session.setMode, {
          sessionId: this.sessionId,
          modeId
        });
      }
    } catch (e) {
      console.warn('[AcpClientSession] setMode failed:', e);
    }
  }

  async setConfigOption(configId: string, value: string): Promise<void> {
    if (!this.connection || !this.sessionId) return;
    try {
      await this.connection.agent.request(acp.methods.agent.session.setConfigOption, {
        sessionId: this.sessionId,
        configId,
        value
      });
    } catch (e) {
      console.warn('[AcpClientSession] setConfigOption failed:', e);
    }
  }

  async sendPrompt(promptText: string, modelId?: string): Promise<void> {
    if (!this.isInitialized || !this.connection) {
      throw new Error('ACP Client Session is not initialized.');
    }

    const params: any = {
      prompt: [{ type: 'text', text: promptText }],
      modelId
    };

    if (this.sessionId) {
      params.sessionId = this.sessionId;
    }

    await this.connection.agent.request(acp.methods.agent.session.prompt, params);
  }
}
