export type AgentStatus = 'idle' | 'working' | 'waiting_for_input' | 'error';
export type TransportType = 'tauri-ipc' | 'websocket' | 'webrtc';

export interface AcpModel {
  id: string;
  name: string;
  provider: string; // Header grouping e.g. "Anthropic (ACP)", "Local (GGUF)", "LMStudio", "Google (API)"
  description?: string;
  contextWindow?: number;
}

export interface ToolCall {
  id: string;
  name: string;
  arguments: Record<string, any>;
  status: 'pending' | 'running' | 'completed' | 'failed';
  result?: string;
  error?: string;
}

export interface ChatMessage {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  thoughts?: string;
  toolCalls?: ToolCall[];
  timestamp: number;
  tokensPerSec?: number;
  latencyMs?: number;
  contextTokens?: number;
  maxContextTokens?: number;
}

export interface AgentConfig {
  id: string;
  name: string;
  description: string;
  transportType: TransportType;
  commandOrUrl: string;
  isDefault?: boolean;
}

export interface SessionData {
  id: string;
  title: string;
  agentId: string;
  agentName: string;
  selectedModelId: string;
  availableModels: AcpModel[];
  status: AgentStatus;
  messages: ChatMessage[];
  createdAt: number;
  updatedAt: number;
  archived?: boolean;
}
