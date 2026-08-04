import type { SessionData, ChatMessage, AgentStatus, AgentConfig, AcpModel, ErrorDetails } from '../types/acp';
import { createMockAcpStream } from '../transports/acp-transport';
import { createTauriIpcStream } from '../transports/tauri-ipc-transport';
import { createWebSocketAcpStream } from '../transports/websocket-transport';
import { AcpClientSession } from '../transports/acp-client-session';

export class SessionStore {
  sessions = $state<SessionData[]>([]);
  activeSessionId = $state<string | null>(null);
  agents = $state<AgentConfig[]>([]);
  autoStartRecording = $state(false);
  private clientSessions = new Map<string, AcpClientSession>();

  constructor() {
    this.agents = [
      { id: 'hirn-local', name: 'Hirn Agent (Local)', description: 'ACP Orchestrator over STDIO', transportType: 'tauri-ipc', commandOrUrl: 'hirn acp', isDefault: true },
      { id: 'goose-local', name: 'Goose Agent', description: 'AAIF Goose CLI', transportType: 'tauri-ipc', commandOrUrl: 'goose acp' },
      { id: 'hirn-serve', name: 'Hirn Server (WS)', description: 'Localhost WebSocket Gateway', transportType: 'websocket', commandOrUrl: 'ws://localhost:3000/acp' }
    ];

    this.sessions = [];
    this.activeSessionId = null;
  }

  get activeSession(): SessionData | null {
    return this.sessions.find(s => s.id === this.activeSessionId) ?? null;
  }

  selectSession(id: string) {
    this.activeSessionId = id;
  }

  archiveSession(id: string) {
    const session = this.sessions.find(s => s.id === id);
    if (session) {
      session.archived = true;
      if (this.activeSessionId === id) {
        const remaining = this.sessions.filter(s => !s.archived);
        this.activeSessionId = remaining[0]?.id ?? null;
      }
    }
  }

  unarchiveSession(id: string) {
    const session = this.sessions.find(s => s.id === id);
    if (session) {
      session.archived = false;
      this.activeSessionId = id;
    }
  }


  createSession(agentId: string, title?: string, eagerConnect = true) {
    const agent = this.agents.find(a => a.id === agentId) ?? this.agents[0];
    let initialModelId = '';
    if (typeof localStorage !== 'undefined') {
      const stored = localStorage.getItem('hirn_selected_model');
      if (stored) initialModelId = stored;
    }
    const newSession: SessionData = {
      id: `sess-${Date.now()}`,
      title: title || `Chat with ${agent.name}`,
      agentId: agent.id,
      agentName: agent.name,
      selectedModelId: initialModelId,
      availableModels: [],
      status: 'idle',
      messages: [],
      createdAt: Date.now(),
      updatedAt: Date.now()
    };
    this.sessions.unshift(newSession);
    this.activeSessionId = newSession.id;

    if (eagerConnect) {
      this.ensureClientSession(newSession.id).catch(err => {
        console.warn('[SessionStore] Eager process connect failed:', err);
      });
    }

    return newSession;
  }

  private async createStreamForAgent(agent: AgentConfig, onError?: (err: any) => void) {
    if (agent.transportType === 'tauri-ipc') {
      return await createTauriIpcStream(agent.commandOrUrl, { onError });
    } else if (agent.transportType === 'websocket') {
      return await createWebSocketAcpStream(agent.commandOrUrl, { onError });
    }
    return createMockAcpStream();
  }

  private async ensureClientSession(sessionId: string): Promise<AcpClientSession> {
    let clientSession = this.clientSessions.get(sessionId);
    if (clientSession) return clientSession;

    const session = this.sessions.find(s => s.id === sessionId);
    const agent = this.agents.find(a => a.id === session?.agentId) || this.agents[0];

    let currentAssistantMsg: ChatMessage | null = null;

    try {
      const stream = await this.createStreamForAgent(agent, (errDetails) => {
        if (!session) return;
        session.status = 'error';
        session.messages.push({
          id: `msg-err-${Date.now()}`,
          role: 'assistant',
          isError: true,
          content: errDetails.message || 'Stream error encountered',
          errorDetails: errDetails,
          timestamp: Date.now()
        });
      });
      clientSession = new AcpClientSession(stream);
      this.clientSessions.set(sessionId, clientSession);

      const result = await clientSession.startSession((notif) => {
        if (!session) return;

        if (notif.type === 'status') {
          session.status = notif.payload;
        } else if (notif.type === 'error') {
          session.status = 'error';
          const p = notif.payload;
          const details: ErrorDetails = (typeof p === 'object' && p !== null) ? {
            source: p.source || 'agent',
            message: p.message || JSON.stringify(p),
            code: p.code,
            method: p.method,
            command: p.command || agent.commandOrUrl,
            pid: p.pid,
            rawPayload: p.rawPayload,
            suggestion: p.suggestion
          } : {
            source: 'agent',
            message: String(p),
            command: agent.commandOrUrl
          };

          const errMsg: ChatMessage = {
            id: `msg-err-${Date.now()}`,
            role: 'assistant',
            isError: true,
            content: details.message,
            errorDetails: details,
            timestamp: Date.now()
          };
          session.messages.push(errMsg);
          currentAssistantMsg = null;
        } else if (notif.type === 'thought') {
          if (!currentAssistantMsg) {
            currentAssistantMsg = {
              id: `msg-${Date.now()}`,
              role: 'assistant',
              content: '',
              thoughts: notif.payload,
              timestamp: Date.now()
            };
            session.messages.push(currentAssistantMsg);
          } else {
            currentAssistantMsg.thoughts = (currentAssistantMsg.thoughts || '') + notif.payload;
          }
        } else if (notif.type === 'tool_call') {
          if (!currentAssistantMsg) {
            currentAssistantMsg = {
              id: `msg-${Date.now()}`,
              role: 'assistant',
              content: '',
              toolCalls: [notif.payload],
              timestamp: Date.now()
            };
            session.messages.push(currentAssistantMsg);
          } else {
            currentAssistantMsg.toolCalls = currentAssistantMsg.toolCalls || [];
            currentAssistantMsg.toolCalls.push(notif.payload);
          }
        } else if (notif.type === 'text') {
          if (!currentAssistantMsg) {
            currentAssistantMsg = {
              id: `msg-${Date.now()}`,
              role: 'assistant',
              content: notif.payload,
              timestamp: Date.now()
            };
            session.messages.push(currentAssistantMsg);
          } else {
            currentAssistantMsg.content = (currentAssistantMsg.content || '') + notif.payload;
          }
        }
      }, session?.cwd);

      if (session && result.models.length > 0) {
        session.availableModels = result.models;
        if (!session.selectedModelId || !session.availableModels.some(m => m.id === session.selectedModelId)) {
          session.selectedModelId = result.models[0].id;
        }
      }
      if (session) {
        session.thoughtLevelConfig = result.thoughtLevelConfig;
      }
    } catch (err: any) {
      console.warn(`[SessionStore] Connection or protocol error for agent ${agent.name}:`, err);
      if (session) {
        session.status = 'error';
        session.messages.push({
          id: `msg-err-${Date.now()}`,
          role: 'assistant',
          isError: true,
          content: `Failed to establish connection with ${agent.name}: ${err?.message || err}`,
          errorDetails: {
            source: 'connection',
            message: err?.message || String(err),
            command: agent.commandOrUrl,
            suggestion: 'Verify that the agent binary is installed and reachable.'
          },
          timestamp: Date.now()
        });
      }
      const fallbackStream = createMockAcpStream();
      clientSession = new AcpClientSession(fallbackStream);
      this.clientSessions.set(sessionId, clientSession);
      const fallbackResult = await clientSession.startSession(() => {}, session?.cwd);
      if (session && fallbackResult.models.length > 0) {
        session.availableModels = fallbackResult.models;
      }
    }

    return clientSession;
  }

  async setThinkingLevel(sessionId: string, value: string) {
    const session = this.sessions.find(s => s.id === sessionId);
    if (!session?.thoughtLevelConfig) return;

    const clientSession = this.clientSessions.get(sessionId);
    if (!clientSession) return;

    await clientSession.setConfigOption(session.thoughtLevelConfig.configId, value);
    session.thoughtLevelConfig = { ...session.thoughtLevelConfig, currentValue: value };
  }

  async sendMessage(sessionId: string, text: string) {
    const session = this.sessions.find(s => s.id === sessionId);
    if (!session) return;
    const agent = this.agents.find(a => a.id === session.agentId) || this.agents[0];

    const userMsg: ChatMessage = {
      id: `msg-${Date.now()}`,
      role: 'user',
      content: text,
      timestamp: Date.now()
    };
    session.messages.push(userMsg);
    session.status = 'working';
    session.updatedAt = Date.now();

    try {
      const clientSession = await this.ensureClientSession(sessionId);
      await clientSession.sendPrompt(text, session.selectedModelId);
    } catch (err: any) {
      session.status = 'error';
      session.messages.push({
        id: `msg-prompt-err-${Date.now()}`,
        role: 'assistant',
        isError: true,
        content: `Prompt Execution Failed: ${err?.message || err}`,
        errorDetails: {
          source: 'jsonrpc',
          message: err?.message || String(err),
          method: 'session/prompt',
          command: agent?.commandOrUrl
        },
        timestamp: Date.now()
      });
    }
  }
}
