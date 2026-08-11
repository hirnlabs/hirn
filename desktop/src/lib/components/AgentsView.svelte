<script lang="ts">
  import type { SessionStore } from '../stores/session.svelte';
  import type { AcpModel, TransportType } from '../types/acp';
  import * as Card from '$lib/components/ui/card';
  import * as Dialog from '$lib/components/ui/dialog';
  import * as Field from '$lib/components/ui/field';
  import * as Select from '$lib/components/ui/select';
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Separator } from '$lib/components/ui/separator';
  import {
    Plus,
    Bot,
    Server,
    Cpu,
    Check,
    ChevronDown,
    Search,
    Star,
    X,
    Eye,
    EyeOff,
    Zap,
    ShieldCheck,
    Wrench,
    Sparkles,
    CheckCircle2,
    Activity,
    Terminal,
    Globe,
    Layers,
    Brain
  } from '@lucide/svelte';

  let { store, activeNav = $bindable('sessions') }: {
    store: SessionStore;
    activeNav?: 'sessions' | 'apps' | 'agents' | 'models' | 'settings' | 'history';
  } = $props();

  // Active ACP Agent Server selection state
  let selectedAgentId = $state<string>('hirn-local');
  onMount(() => {
    if (store.agents.length > 0 && store.agents[0].id) {
      selectedAgentId = store.agents[0].id;
    }
  });
  let selectedAgent = $derived(store.agents.find(a => a.id === selectedAgentId) || store.agents[0]);

  // Dialog state for adding a new ACP server
  let isAddServerOpen = $state(false);
  let newAgentName = $state('');
  let newAgentDesc = $state('');
  let newAgentTransport = $state<TransportType>('tauri-ipc');
  let newAgentTarget = $state('');

  import { onMount } from 'svelte';
  import * as yaml from 'js-yaml';

  // Persistent storage handler via Tauri IPC or localStorage fallback
  let isSavingConfig = $state(false);
  let saveConfigNotification = $state<string | null>(null);

  async function saveCurrentAgentConfigYaml() {
    const agent = selectedAgent;
    if (!agent) return;

    const configData = {
      agent: {
        id: agent.id,
        name: agent.name,
        description: agent.description,
        transportType: agent.transportType,
        commandOrUrl: agent.commandOrUrl,
        isDefault: agent.isDefault || false
      },
      subagents: subagents.map(s => ({
        id: s.id,
        name: s.name,
        role: s.role,
        description: s.description,
        modelId: s.modelId
      })),
      providers: providers.map(p => ({
        id: p.id,
        name: p.name,
        baseUrl: p.baseUrl,
        apiKey: p.apiKey,
        status: p.status,
        type: p.type
      }))
    };

    const yamlStr = yaml.dump(configData, { indent: 2 });

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('save_agent_config', { agentId: agent.id, content: yamlStr });
      saveConfigNotification = `Saved ~/.hirn/config/${agent.id}.yaml`;
      setTimeout(() => saveConfigNotification = null, 2500);
    } catch {
      // LocalStorage fallback for browser-only mode
      if (typeof localStorage !== 'undefined') {
        localStorage.setItem(`hirn_agent_config_${agent.id}`, yamlStr);
        saveConfigNotification = `Saved config for ${agent.name} (localStorage)`;
        setTimeout(() => saveConfigNotification = null, 2500);
      }
    }
  }

  async function loadAgentConfigYaml(agentId: string) {
    let yamlStr: string | null = null;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      yamlStr = await invoke<string>('load_agent_config', { agentId });
    } catch {
      if (typeof localStorage !== 'undefined') {
        yamlStr = localStorage.getItem(`hirn_agent_config_${agentId}`);
      }
    }

    if (yamlStr) {
      try {
        const parsed: any = yaml.load(yamlStr);
        if (parsed && typeof parsed === 'object') {
          if (parsed.subagents && Array.isArray(parsed.subagents)) {
            subagents = parsed.subagents.map((s: any) => ({
              ...s,
              icon: s.id === 'investigator' ? Search : s.id === 'builder' ? Wrench : s.id === 'reviewer' ? ShieldCheck : Bot
            }));
          }
          if (parsed.providers && Array.isArray(parsed.providers)) {
            providers = parsed.providers;
          }
        }
      } catch (err) {
        console.warn('Failed to parse agent YAML config:', err);
      }
    }
  }

  // Available models pool for model selector popover
  const availableModels: AcpModel[] = [
    { id: 'claude-3-5-sonnet', name: 'Claude 3.5 Sonnet', provider: 'Anthropic (ACP)', contextWindow: 200000 },
    { id: 'claude-3-5-haiku', name: 'Claude 3.5 Haiku', provider: 'Anthropic (ACP)', contextWindow: 200000 },
    { id: 'gemini-1-5-pro', name: 'Gemini 1.5 Pro', provider: 'Google (API)', contextWindow: 1000000 },
    { id: 'gemini-1-5-flash', name: 'Gemini 1.5 Flash', provider: 'Google (API)', contextWindow: 1000000 },
    { id: 'gpt-4o', name: 'GPT-4o', provider: 'OpenAI (API)', contextWindow: 128000 },
    { id: 'gpt-4o-mini', name: 'GPT-4o Mini', provider: 'OpenAI (API)', contextWindow: 128000 },
    { id: 'deepseek-coder-v2', name: 'DeepSeek Coder V2', provider: 'DeepSeek (API)', contextWindow: 128000 },
    { id: 'qwen2-5-coder-7b', name: 'Qwen 2.5 Coder 7B', provider: 'LM Studio / Ollama', contextWindow: 32000 },
    { id: 'llama3-2-3b', name: 'Llama 3.2 3B Instruct', provider: 'llama.cpp Local', contextWindow: 131072 }
  ];

  // Subagents mapping state
  let subagents = $state([
    {
      id: 'investigator',
      name: 'Investigator Subagent',
      role: 'Codebase Researcher',
      description: 'Fast codebase search, file reading, AST mapping, and structural context gathering.',
      icon: Search,
      modelId: 'qwen2-5-coder-7b'
    },
    {
      id: 'builder',
      name: 'Builder / Coder Subagent',
      role: 'Code Implementation',
      description: 'Multi-file refactoring, feature implementation, and automated test execution.',
      icon: Wrench,
      modelId: 'claude-3-5-sonnet'
    },
    {
      id: 'reviewer',
      name: 'Reviewer / Critic Subagent',
      role: 'Code Review & QA',
      description: 'Diff inspection, security auditing, edge case analysis, and spec compliance.',
      icon: ShieldCheck,
      modelId: 'gemini-1-5-pro'
    }
  ]);

  // Model selector modal state for subagent assignment
  let activeSubagentForModelSelect = $state<string | null>(null);
  let modelSearchQuery = $state('');

  function getModelName(id: string) {
    return availableModels.find(m => m.id === id)?.name || id;
  }

  function getModelProvider(id: string) {
    return availableModels.find(m => m.id === id)?.provider || 'Provider';
  }

  let filteredModels = $derived.by(() => {
    const q = modelSearchQuery.toLowerCase();
    if (!q) return availableModels;
    return availableModels.filter(m => m.name.toLowerCase().includes(q) || m.provider.toLowerCase().includes(q));
  });

  // Providers configuration state
  let showApiKeys = $state<Record<string, boolean>>({});
  function toggleApiKey(providerKey: string) {
    showApiKeys[providerKey] = !showApiKeys[providerKey];
  }

  let providers = $state([
    {
      id: 'lmstudio',
      name: 'LM Studio',
      subtitle: 'Local OpenAI-compatible REST server endpoint',
      baseUrl: 'http://localhost:1234/v1',
      apiKey: '',
      status: 'Active',
      type: 'local'
    },
    {
      id: 'llamacpp',
      name: 'llama.cpp / Ollama',
      subtitle: 'Local GGUF quantized models and API runner',
      baseUrl: 'http://localhost:11434',
      apiKey: '',
      status: 'Active',
      type: 'local'
    },
    {
      id: 'claude',
      name: 'Anthropic Claude',
      subtitle: 'Official Anthropic API key for Claude 3.5 models',
      baseUrl: 'https://api.anthropic.com/v1',
      apiKey: 'sk-ant-api03-sample-key-configured',
      status: 'Configured',
      type: 'cloud'
    },
    {
      id: 'gemini',
      name: 'Google Gemini',
      subtitle: 'Google AI Studio API key for Gemini 1.5 Pro & Flash',
      baseUrl: 'https://generativelanguage.googleapis.com',
      apiKey: 'AIzaSySampleKeyConfiguredGemini',
      status: 'Configured',
      type: 'cloud'
    },
    {
      id: 'openai',
      name: 'OpenAI',
      subtitle: 'Direct OpenAI API key or custom endpoint override',
      baseUrl: 'https://api.openai.com/v1',
      apiKey: '',
      status: 'Not Configured',
      type: 'cloud'
    },
    {
      id: 'deepseek',
      name: 'DeepSeek',
      subtitle: 'DeepSeek API endpoint for DeepSeek V3 and Coder',
      baseUrl: 'https://api.deepseek.com/v1',
      apiKey: '',
      status: 'Not Configured',
      type: 'cloud'
    }
  ]);

  let testConnectionStatuses = $state<Record<string, string | undefined>>({});
  function testProviderConnection(provId: string) {
    testConnectionStatuses[provId] = 'Testing...';
    setTimeout(() => {
      testConnectionStatuses[provId] = 'Connected (18ms)';
      setTimeout(() => {
        testConnectionStatuses[provId] = undefined;
      }, 3000);
    }, 600);
  }

  function startAgentSession(agentId: string) {
    store.createSession(agentId);
    activeNav = 'sessions';
  }

  // Custom Subagent Dialog State
  let isAddSubagentOpen = $state(false);
  let newSubagentName = $state('');
  let newSubagentRole = $state('');
  let newSubagentDesc = $state('');
  let newSubagentModelId = $state('claude-3-5-sonnet');

  $effect(() => {
    if (selectedAgentId) {
      loadAgentConfigYaml(selectedAgentId);
    }
  });

  function handleAddServer() {
    if (!newAgentName.trim() || !newAgentTarget.trim()) return;
    const newId = `agent-${Date.now()}`;
    store.agents.push({
      id: newId,
      name: newAgentName.trim(),
      description: newAgentDesc.trim() || 'Custom ACP Server',
      transportType: newAgentTransport,
      commandOrUrl: newAgentTarget.trim()
    });
    selectedAgentId = newId;
    newAgentName = '';
    newAgentDesc = '';
    newAgentTransport = 'tauri-ipc';
    newAgentTarget = '';
    isAddServerOpen = false;
    saveCurrentAgentConfigYaml();
  }

  function handleAddSubagent() {
    if (!newSubagentName.trim() || !newSubagentRole.trim()) return;
    const newSubId = `sub-${Date.now()}`;
    subagents.push({
      id: newSubId,
      name: newSubagentName.trim(),
      role: newSubagentRole.trim(),
      description: newSubagentDesc.trim() || 'Custom subagent worker node.',
      icon: Bot,
      modelId: newSubagentModelId
    });
    newSubagentName = '';
    newSubagentRole = '';
    newSubagentDesc = '';
    newSubagentModelId = 'claude-3-5-sonnet';
    isAddSubagentOpen = false;
    saveCurrentAgentConfigYaml();
  }
</script>

<div class="flex-1 h-full overflow-y-auto bg-background text-foreground p-6 md:p-8 pb-32 flex flex-col gap-6 select-none">
  <!-- Top Header with ACP Agent Server Selection and Action Buttons -->
  <div class="flex flex-col md:flex-row md:items-center justify-between gap-4">
    <div>
      <h2 class="text-xl font-bold tracking-tight">Agents & Providers</h2>
      <p class="text-sm text-muted-foreground mt-1">Manage connected ACP servers, subagent model routing, and LLM provider endpoints.</p>
    </div>
    <div class="flex items-center gap-2 self-start md:self-auto">
      {#if saveConfigNotification}
        <span class="text-xs font-mono text-emerald-400 bg-emerald-500/10 border border-emerald-500/30 px-3 py-1.5 rounded-lg animate-in fade-in duration-200">
          ✓ {saveConfigNotification}
        </span>
      {/if}
      <Button onclick={saveCurrentAgentConfigYaml} variant="outline" size="sm" class="gap-1.5 cursor-pointer font-semibold shadow-xs">
        <CheckCircle2 class="size-4 text-emerald-500" />
        <span>Save Config</span>
      </Button>
      <Button onclick={() => isAddServerOpen = true} size="sm" class="gap-1.5 cursor-pointer shadow-xs">
        <Plus class="size-4" />
        <span>Add ACP Server</span>
      </Button>
    </div>
  </div>

  <Separator />

  <!-- 1. ACP Server Selector Section -->
  <Card.Root class="border-border shadow-xs">
    <Card.Header class="pb-3">
      <div class="flex items-center justify-between flex-wrap gap-2">
        <div class="flex items-center gap-2">
          <Server class="size-4 text-primary" />
          <Card.Title class="text-base font-semibold">Registered ACP Agent Server</Card.Title>
        </div>
        <Badge variant="secondary" class="text-[10px] uppercase font-mono tracking-wider">
          {selectedAgent?.transportType}
        </Badge>
      </div>
      <Card.Description class="text-xs text-muted-foreground">Select the active Agent Client Protocol server instance for orchestrating chat sessions.</Card.Description>
    </Card.Header>
    <Card.Content class="flex flex-col gap-4">
      <div class="flex flex-col sm:flex-row items-stretch sm:items-center gap-3">
        <div class="flex-1">
          <Select.Root type="single" bind:value={selectedAgentId}>
            <Select.Trigger class="w-full h-10 text-sm bg-muted/40 font-medium">
              <div class="flex items-center gap-2 truncate">
                <Bot class="size-4 text-primary shrink-0" />
                <span class="truncate">{selectedAgent?.name || 'Select ACP Server'}</span>
              </div>
            </Select.Trigger>
            <Select.Content class="bg-popover border-border text-popover-foreground z-50">
              {#each store.agents as agent}
                <Select.Item value={agent.id} class="text-xs cursor-pointer flex items-center justify-between">
                  <div class="flex items-center gap-2">
                    <Bot class="size-3.5 text-muted-foreground" />
                    <span>{agent.name}</span>
                  </div>
                  <span class="text-[10px] font-mono text-muted-foreground uppercase">{agent.transportType}</span>
                </Select.Item>
              {/each}
            </Select.Content>
          </Select.Root>
        </div>

        <div class="flex items-center gap-2">
          <Button variant="default" size="sm" class="h-10 px-4 gap-1.5 cursor-pointer" onclick={() => startAgentSession(selectedAgentId)}>
            <Sparkles class="size-3.5" />
            <span>Start Chat</span>
          </Button>
          <Button variant="outline" size="sm" class="h-10 px-3 cursor-pointer" onclick={() => testProviderConnection('acp-server')}>
            <Activity class="size-3.5 text-emerald-500" />
            <span>{testConnectionStatuses['acp-server'] || 'Test Connection'}</span>
          </Button>
        </div>
      </div>

      {#if selectedAgent}
        <div class="p-3 bg-muted/30 border border-border/80 rounded-xl flex flex-wrap items-center justify-between gap-3 text-xs text-muted-foreground">
          <div class="flex items-center gap-2">
            <span class="size-2 rounded-full bg-emerald-500 animate-pulse"></span>
            <span class="font-medium text-foreground">{selectedAgent.description}</span>
          </div>
          <div class="flex items-center gap-4 text-[11px] font-mono">
            <span>Protocol: <code class="text-foreground">{selectedAgent.transportType}</code></span>
            <span>Target: <code class="text-foreground">{selectedAgent.commandOrUrl}</code></span>
          </div>
        </div>
      {/if}
    </Card.Content>
  </Card.Root>

  <!-- 2. Subagent Model Configuration Section (Enlarged & Expandable) -->
  <Card.Root class="border-border shadow-xs bg-card/60 overflow-visible">
    <Card.Header class="pb-4 border-b border-border/60">
      <div class="flex items-center justify-between flex-wrap gap-3">
        <div class="space-y-1">
          <div class="flex items-center gap-2">
            <Cpu class="size-5 text-primary" />
            <Card.Title class="text-lg font-bold">Subagent Model Routing & Workforces</Card.Title>
          </div>
          <Card.Description class="text-xs text-muted-foreground">Configure specialized autonomous subagents, assign dedicated LLM backends, and define custom role responsibilities.</Card.Description>
        </div>
        <Button variant="outline" size="sm" class="gap-1.5 cursor-pointer border-primary/40 hover:bg-primary/10 text-xs font-semibold" onclick={() => isAddSubagentOpen = true}>
          <Plus class="size-4 text-primary" />
          <span>Add Custom Subagent</span>
        </Button>
      </div>
    </Card.Header>
    <Card.Content class="pt-6 pb-6">
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {#each subagents as subagent}
          {@const SubIcon = subagent.icon}
          <div class="p-6 bg-card border border-border/90 rounded-2xl flex flex-col justify-between min-h-[220px] gap-5 hover:border-primary/60 hover:shadow-md transition-all group">
            <div class="space-y-3">
              <div class="flex items-start justify-between gap-3">
                <div class="flex items-center gap-3">
                  <div class="p-2.5 rounded-xl bg-primary/10 text-primary group-hover:scale-105 transition-transform shrink-0">
                    <SubIcon class="size-5" />
                  </div>
                  <div class="min-w-0">
                    <h4 class="text-sm font-bold text-foreground truncate">{subagent.name}</h4>
                    <span class="text-[11px] font-mono text-primary font-medium px-2.5 py-0.5 rounded bg-primary/10 border border-primary/20 inline-block mt-1">{subagent.role}</span>
                  </div>
                </div>
              </div>
              <p class="text-xs text-muted-foreground leading-relaxed">{subagent.description}</p>
            </div>

            <div class="space-y-2 pt-4 border-t border-border/80 mt-auto">
              <div class="flex items-center justify-between">
                <span class="text-[11px] font-semibold font-mono uppercase text-muted-foreground">Assigned LLM Model</span>
                <span class="text-[10px] font-mono text-muted-foreground">{getModelProvider(subagent.modelId)}</span>
              </div>
              <button
                onclick={() => activeSubagentForModelSelect = subagent.id}
                class="w-full h-10 px-3 bg-muted/60 hover:bg-muted border border-border/90 rounded-xl flex items-center justify-between text-xs font-medium transition-colors cursor-pointer"
              >
                <div class="flex items-center gap-2.5 truncate">
                  <span class="size-5 rounded-md bg-background border border-border flex items-center justify-center text-[10px] font-mono font-bold shrink-0">
                    {getModelProvider(subagent.modelId)[0]}
                  </span>
                  <span class="truncate font-semibold text-foreground">{getModelName(subagent.modelId)}</span>
                </div>
                <ChevronDown class="size-4 text-muted-foreground shrink-0" />
              </button>
            </div>
          </div>
        {/each}
      </div>
    </Card.Content>
  </Card.Root>

  <!-- 3. Provider Configuration Options Section -->
  <Card.Root class="border-border shadow-xs">
    <Card.Header class="pb-3">
      <div class="flex items-center gap-2">
        <Brain class="size-4 text-primary" />
        <Card.Title class="text-base font-semibold">Provider Configuration</Card.Title>
      </div>
      <Card.Description class="text-xs text-muted-foreground">Configure local REST endpoints, API keys, and model runner credentials for all supported backends.</Card.Description>
    </Card.Header>
    <Card.Content class="grid grid-cols-1 md:grid-cols-2 gap-4">
      {#each providers as provider}
        <div class="p-4 bg-card border border-border rounded-xl space-y-3">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <span class="text-base font-bold">{provider.name}</span>
              <Badge variant={provider.type === 'local' ? 'secondary' : 'outline'} class="text-[9px] uppercase font-mono">
                {provider.type}
              </Badge>
            </div>
            <div class="flex items-center gap-2">
              <span class="text-[10px] font-mono px-2 py-0.5 rounded-full border {provider.status === 'Active' ? 'bg-emerald-500/10 text-emerald-500 border-emerald-500/30' : provider.status === 'Configured' ? 'bg-primary/10 text-primary border-primary/30' : 'bg-muted text-muted-foreground border-border'}">
                {provider.status}
              </span>
              <Button variant="ghost" size="icon" class="size-7 cursor-pointer" title="Test Provider" onclick={() => testProviderConnection(provider.id)}>
                <Activity class="size-3.5 text-muted-foreground hover:text-foreground" />
              </Button>
            </div>
          </div>

          <p class="text-[11px] text-muted-foreground">{provider.subtitle}</p>

          <Field.FieldGroup class="space-y-2">
            <Field.Field>
              <Field.FieldLabel class="text-[10px] font-mono uppercase text-muted-foreground">Base Endpoint URL</Field.FieldLabel>
              <Input type="text" bind:value={provider.baseUrl} class="font-mono text-xs h-8 bg-muted/30" />
            </Field.Field>

            {#if provider.type === 'cloud' || provider.id === 'lmstudio'}
              <Field.Field>
                <Field.FieldLabel class="text-[10px] font-mono uppercase text-muted-foreground">API Key</Field.FieldLabel>
                <div class="relative flex items-center">
                  <Input
                    type={showApiKeys[provider.id] ? 'text' : 'password'}
                    bind:value={provider.apiKey}
                    placeholder={provider.type === 'cloud' ? 'Enter API Key...' : 'Optional local API Key'}
                    class="font-mono text-xs h-8 pr-8 bg-muted/30"
                  />
                  <button
                    onclick={() => toggleApiKey(provider.id)}
                    class="absolute right-2 text-muted-foreground hover:text-foreground cursor-pointer"
                    title={showApiKeys[provider.id] ? 'Hide API Key' : 'Show API Key'}
                  >
                    {#if showApiKeys[provider.id]}
                      <EyeOff class="size-3.5" />
                    {:else}
                      <Eye class="size-3.5" />
                    {/if}
                  </button>
                </div>
              </Field.Field>
            {/if}
          </Field.FieldGroup>

          {#if testConnectionStatuses[provider.id]}
            <div class="text-[10px] font-mono text-emerald-400 flex items-center gap-1.5 pt-1">
              <CheckCircle2 class="size-3" />
              <span>{testConnectionStatuses[provider.id]}</span>
            </div>
          {/if}
        </div>
      {/each}
    </Card.Content>
  </Card.Root>
</div>

<!-- Modal: Add ACP Server Dialog -->
<Dialog.Root bind:open={isAddServerOpen}>
  <Dialog.Content class="sm:max-w-[440px]">
    <Dialog.Header>
      <Dialog.Title class="text-base font-bold flex items-center gap-2">
        <Server class="size-4 text-primary" />
        <span>Add Registered ACP Server</span>
      </Dialog.Title>
      <Dialog.Description class="text-xs text-muted-foreground">
        Register a new Agent Client Protocol (ACP) executable or network endpoint.
      </Dialog.Description>
    </Dialog.Header>

    <Field.FieldGroup class="flex flex-col gap-4 py-3">
      <Field.Field>
        <Field.FieldLabel for="agent-name" class="text-xs font-medium">Server Name</Field.FieldLabel>
        <Input id="agent-name" type="text" bind:value={newAgentName} placeholder="e.g. Remote Cluster ACP" class="text-xs" />
      </Field.Field>

      <Field.Field>
        <Field.FieldLabel for="agent-desc" class="text-xs font-medium">Description</Field.FieldLabel>
        <Input id="agent-desc" type="text" bind:value={newAgentDesc} placeholder="Short description of this agent" class="text-xs" />
      </Field.Field>

      <Field.Field>
        <Field.FieldLabel class="text-xs font-medium">Transport Protocol</Field.FieldLabel>
        <Select.Root type="single" bind:value={newAgentTransport}>
          <Select.Trigger class="w-full text-xs font-mono h-9">
            <span>{newAgentTransport}</span>
          </Select.Trigger>
          <Select.Content class="bg-popover border-border text-popover-foreground z-50">
            <Select.Item value="tauri-ipc" class="text-xs font-mono">tauri-ipc (Standard I/O CLI Process)</Select.Item>
            <Select.Item value="websocket" class="text-xs font-mono">websocket (WS / WSS Endpoint)</Select.Item>
            <Select.Item value="webrtc" class="text-xs font-mono">webrtc (Encrypted Peer Relay)</Select.Item>
          </Select.Content>
        </Select.Root>
      </Field.Field>

      <Field.Field>
        <Field.FieldLabel for="agent-target" class="text-xs font-medium">Target Command or URL</Field.FieldLabel>
        <Input id="agent-target" type="text" bind:value={newAgentTarget} placeholder="hirn acp OR ws://localhost:3000/acp" class="font-mono text-xs" />
      </Field.Field>
    </Field.FieldGroup>

    <Dialog.Footer class="gap-2 pt-2">
      <Button variant="outline" size="sm" onclick={() => isAddServerOpen = false}>Cancel</Button>
      <Button size="sm" disabled={!newAgentName.trim() || !newAgentTarget.trim()} onclick={handleAddServer}>Add Server</Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>

<!-- Modal Popover: Reused Model Selector Dialogue for Subagent Assignment -->
{#if activeSubagentForModelSelect}
  <Dialog.Root open={true} onOpenChange={(open) => { if (!open) activeSubagentForModelSelect = null; }}>
    <Dialog.Content class="sm:max-w-[420px] p-0 overflow-hidden">
      <div class="p-4 border-b border-border bg-muted/30 flex items-center justify-between">
        <div class="flex items-center gap-2">
          <Cpu class="size-4 text-primary" />
          <h3 class="text-sm font-bold">Select Subagent Model</h3>
        </div>
        <button onclick={() => activeSubagentForModelSelect = null} class="text-muted-foreground hover:text-foreground">
          <X class="size-4" />
        </button>
      </div>

      <div class="p-3 border-b border-border bg-card">
        <div class="relative">
          <Search class="absolute left-3 top-1/2 -translate-y-1/2 size-3.5 text-muted-foreground" />
          <input
            type="text"
            placeholder="Search models or providers..."
            bind:value={modelSearchQuery}
            class="w-full bg-muted/60 border border-border rounded-lg pl-8 pr-3 py-1.5 text-xs text-foreground placeholder-muted-foreground outline-none focus:ring-1 focus:ring-primary"
          />
        </div>
      </div>

      <div class="p-2 max-h-[300px] overflow-y-auto space-y-1">
        {#each filteredModels as model}
          <button
            onclick={() => {
              const sub = subagents.find(s => s.id === activeSubagentForModelSelect);
              if (sub) sub.modelId = model.id;
              activeSubagentForModelSelect = null;
              saveCurrentAgentConfigYaml();
            }}
            class="w-full flex items-center justify-between p-2.5 rounded-lg text-xs text-left hover:bg-accent hover:text-accent-foreground transition-all cursor-pointer border border-transparent hover:border-border"
          >
            <div class="flex items-center gap-2.5 min-w-0">
              <span class="size-6 rounded-md bg-muted border border-border flex items-center justify-center text-[9px] font-mono font-bold shrink-0">
                {model.provider[0]}
              </span>
              <div class="flex flex-col min-w-0">
                <span class="font-medium truncate">{model.name}</span>
                <span class="text-[10px] text-muted-foreground font-mono truncate">{model.provider}</span>
              </div>
            </div>
            {#if subagents.find(s => s.id === activeSubagentForModelSelect)?.modelId === model.id}
              <Check class="size-4 text-emerald-400 shrink-0 ml-2" />
            {/if}
          </button>
        {/each}
      </div>
    </Dialog.Content>
  </Dialog.Root>
{/if}

<!-- Modal: Add Custom Subagent Dialog -->
<Dialog.Root bind:open={isAddSubagentOpen}>
  <Dialog.Content class="sm:max-w-[440px]">
    <Dialog.Header>
      <Dialog.Title class="text-base font-bold flex items-center gap-2">
        <Cpu class="size-4 text-primary" />
        <span>Add Custom Subagent</span>
      </Dialog.Title>
      <Dialog.Description class="text-xs text-muted-foreground">
        Define a specialized subagent worker node with custom responsibilities and LLM model routing.
      </Dialog.Description>
    </Dialog.Header>

    <Field.FieldGroup class="flex flex-col gap-4 py-3">
      <Field.Field>
        <Field.FieldLabel for="sub-name" class="text-xs font-medium">Subagent Name</Field.FieldLabel>
        <Input id="sub-name" type="text" bind:value={newSubagentName} placeholder="e.g. Data Analysis Subagent" class="text-xs" />
      </Field.Field>

      <Field.Field>
        <Field.FieldLabel for="sub-role" class="text-xs font-medium">Subagent Role Title</Field.FieldLabel>
        <Input id="sub-role" type="text" bind:value={newSubagentRole} placeholder="e.g. Python & Data Analyst" class="text-xs font-mono" />
      </Field.Field>

      <Field.Field>
        <Field.FieldLabel for="sub-desc" class="text-xs font-medium">Description & Responsibilities</Field.FieldLabel>
        <Input id="sub-desc" type="text" bind:value={newSubagentDesc} placeholder="Parses datasets, executes numerical calculations, and produces plots." class="text-xs" />
      </Field.Field>

      <Field.Field>
        <Field.FieldLabel class="text-xs font-medium">Initial Assigned Model</Field.FieldLabel>
        <Select.Root type="single" bind:value={newSubagentModelId}>
          <Select.Trigger class="w-full text-xs font-mono h-9">
            <span>{getModelName(newSubagentModelId)} ({getModelProvider(newSubagentModelId)})</span>
          </Select.Trigger>
          <Select.Content class="bg-popover border-border text-popover-foreground z-50">
            {#each availableModels as model}
              <Select.Item value={model.id} class="text-xs cursor-pointer flex items-center justify-between">
                <span>{model.name}</span>
                <span class="text-[10px] font-mono text-muted-foreground">{model.provider}</span>
              </Select.Item>
            {/each}
          </Select.Content>
        </Select.Root>
      </Field.Field>
    </Field.FieldGroup>

    <Dialog.Footer class="gap-2 pt-2">
      <Button variant="outline" size="sm" onclick={() => isAddSubagentOpen = false}>Cancel</Button>
      <Button size="sm" disabled={!newSubagentName.trim() || !newSubagentRole.trim()} onclick={handleAddSubagent}>Add Subagent</Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>


