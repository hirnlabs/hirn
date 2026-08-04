export type AgentStatus = 'idle' | 'working' | 'waiting_for_input' | 'error';
export type TransportType = 'tauri-ipc' | 'websocket' | 'webrtc';

export interface AcpModel {
  id: string;
  name: string;
  provider: string; // Header grouping e.g. "Anthropic (ACP)", "Local (GGUF)", "LMStudio", "Google (API)"
  description?: string;
  contextWindow?: number;
  thinking?: {
    supported: boolean;
    levels?: ('off' | 'low' | 'mid' | 'high' | 'max')[];
  };
}

/** Parsed from ACP configOptions where category === 'thought_level' */
export interface ThoughtLevelConfig {
  configId: string;
  currentValue: string;
  levels: { value: string; name: string }[];
}

export interface ToolCall {
  id: string;
  name: string;
  arguments: Record<string, any>;
  status: 'pending' | 'running' | 'completed' | 'failed';
  result?: string;
  error?: string;
}

export interface ErrorDetails {
  source: 'stdio' | 'jsonrpc' | 'ipc' | 'connection' | 'agent';
  message: string;
  code?: number | string;
  method?: string;
  command?: string;
  pid?: number | null;
  rawPayload?: string;
  suggestion?: string;
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
  isError?: boolean;
  errorDetails?: ErrorDetails;
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
  cwd?: string;
  /** Populated from ACP configOptions with category 'thought_level'. Null = agent doesn't support it. */
  thoughtLevelConfig?: ThoughtLevelConfig | null;
}
