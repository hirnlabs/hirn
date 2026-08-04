import type { SessionData, ChatMessage, AgentStatus, AgentConfig, AcpModel } from '../types/acp';
import { MockAcpTransport, type AcpTransport } from '../transports/acp-transport';
import { TauriIpcTransport } from '../transports/tauri-ipc-transport';
import { WebSocketAcpTransport } from '../transports/websocket-transport';
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

    const defaultModels: AcpModel[] = [
      { id: 'local/gemma-4-9b-it', name: 'gemma-4-9b-it', provider: 'Local (GGUF)' },
      { id: 'local/qwen2.5-coder-7b', name: 'qwen2.5-coder-7b', provider: 'Local (GGUF)' },
      { id: 'anthropic/claude-3-5-sonnet', name: 'claude-3.5-sonnet', provider: 'Anthropic (ACP)' },
      { id: 'anthropic/claude-3-7-sonnet', name: 'claude-3.7-sonnet', provider: 'Anthropic (ACP)' },
      { id: 'lmstudio/llama3.2-3b-instruct', name: 'llama3.2-3b-instruct', provider: 'LMStudio' },
      { id: 'google/gemini-2.5-flash', name: 'gemini-2.5-flash', provider: 'Google (API)' }
    ];

    this.sessions = [
      {
        id: 'sess-1',
        title: 'Desktop Architecture & State Store',
        agentId: 'hirn-local',
        agentName: 'Hirn Agent (Local)',
        selectedModelId: 'local/gemma-4-9b-it',
        availableModels: defaultModels,
        status: 'idle',
        createdAt: Date.now() - 3600000,
        updatedAt: Date.now() - 1800000,
        messages: [
          { id: 'm1', role: 'user', content: 'Design the Svelte 5 Runes state model for Desktop.', timestamp: Date.now() - 3600000 },
          {
            id: 'm2',
            role: 'assistant',
            content: 'I have designed the `SessionStore` class model with reactive `$state` fields.',
            thoughts: 'Checking Svelte 5 runes documentation and local persistence contracts.',
            toolCalls: [
              { id: 't1', name: 'write_file', arguments: { path: 'src/lib/stores/session.svelte.ts' }, status: 'completed', result: 'File created successfully.' }
            ],
            timestamp: Date.now() - 3500000
          }
        ]
      }
    ];

    this.activeSessionId = 'sess-1';
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


  createSession(agentId: string, title?: string) {
    const agent = this.agents.find(a => a.id === agentId) ?? this.agents[0];
    const defaultModels: AcpModel[] = [
      { id: 'local/gemma-4-9b-it', name: 'gemma-4-9b-it', provider: 'Local (GGUF)' },
      { id: 'local/qwen2.5-coder-7b', name: 'qwen2.5-coder-7b', provider: 'Local (GGUF)' },
      { id: 'anthropic/claude-3-5-sonnet', name: 'claude-3.5-sonnet', provider: 'Anthropic (ACP)' },
      { id: 'anthropic/claude-3-7-sonnet', name: 'claude-3.7-sonnet', provider: 'Anthropic (ACP)' },
      { id: 'lmstudio/llama3.2-3b-instruct', name: 'llama3.2-3b-instruct', provider: 'LMStudio' },
      { id: 'google/gemini-2.5-flash', name: 'gemini-2.5-flash', provider: 'Google (API)' }
    ];
    const newSession: SessionData = {
      id: `sess-${Date.now()}`,
      title: title || `Chat with ${agent.name}`,
      agentId: agent.id,
      agentName: agent.name,
      selectedModelId: 'local/gemma-4-9b-it',
      availableModels: defaultModels,
      status: 'idle',
      messages: [],
      createdAt: Date.now(),
      updatedAt: Date.now()
    };
    this.sessions.unshift(newSession);
    this.activeSessionId = newSession.id;
    return newSession;
  }

  private createTransportForAgent(agent: AgentConfig): AcpTransport {
    if (agent.transportType === 'tauri-ipc') {
      return new TauriIpcTransport(agent.commandOrUrl);
    } else if (agent.transportType === 'websocket') {
      return new WebSocketAcpTransport(agent.commandOrUrl);
    }
    return new MockAcpTransport();
  }

  async sendMessage(sessionId: string, text: string) {
    const session = this.sessions.find(s => s.id === sessionId);
    if (!session) return;

    const userMsg: ChatMessage = {
      id: `msg-${Date.now()}`,
      role: 'user',
      content: text,
      timestamp: Date.now()
    };
    session.messages.push(userMsg);
    session.status = 'working';
    session.updatedAt = Date.now();

    let clientSession = this.clientSessions.get(sessionId);
    if (!clientSession) {
      const agent = this.agents.find(a => a.id === session.agentId) || this.agents[0];
      const transport = this.createTransportForAgent(agent);
      clientSession = new AcpClientSession(transport);
      this.clientSessions.set(sessionId, clientSession);

      let currentAssistantMsg: ChatMessage | null = null;

      try {
        const fetchedModels = await clientSession.startSession((notif) => {
          if (notif.type === 'status') {
            session.status = notif.payload;
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
              currentAssistantMsg.thoughts = notif.payload;
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
              currentAssistantMsg.content = notif.payload;
            }
          }
        });

        if (fetchedModels && fetchedModels.length > 0) {
          session.availableModels = fetchedModels;
        }
      } catch (err) {
        console.warn(`[SessionStore] Failed to connect to agent ${agent.name}, falling back to mock transport:`, err);
        const fallbackTransport = new MockAcpTransport();
        clientSession = new AcpClientSession(fallbackTransport);
        this.clientSessions.set(sessionId, clientSession);
        const fetchedModels = await clientSession.startSession(() => {});
        if (fetchedModels && fetchedModels.length > 0) {
          session.availableModels = fetchedModels;
        }
      }
    }

    await clientSession.sendPrompt(text, session.selectedModelId);
  }
}
