<script lang="ts">
  import type { SessionStore } from '../stores/session.svelte';
  import type { AcpModel } from '../types/acp';
  import ToolExecutionCard from './ToolExecutionCard.svelte';
  import * as InputGroup from './ui/input-group';
  import * as DropdownMenu from './ui/dropdown-menu';
  import { Separator } from './ui/separator';
  import { 
    ChevronDown, 
    Paperclip, 
    X, 
    Search, 
    Star, 
    Check, 
    Compass,
    Wrench,
    Sparkles,
    Play,
    Plus as IconPlus,
    ArrowUp as ArrowUpIcon,
    File,
    Folder
  } from '@lucide/svelte';

  let { store }: { store: SessionStore } = $props();
  let promptText = $state('');
  let isModelMenuOpen = $state(false);
  let isToolsMenuOpen = $state(false);
  let isAttachMenuOpen = $state(false);
  let searchQuery = $state('');
  let toolsSearchQuery = $state('');

  // File & Folder Attachments State
  let fileInputRef = $state<HTMLInputElement | null>(null);
  let folderInputRef = $state<HTMLInputElement | null>(null);
  let attachedFiles = $state<{ name: string; path: string; isFolder?: boolean }[]>([]);

  let activeSession = $derived(store.activeSession);

  // Local Storage Persistence for Favorites and Selected Model
  let favoriteModelIds = $state<string[]>(loadFavorites());

  // 3-State & 4-State Permission Control System
  export type PermissionState = 'off' | 'ask' | 'allow';
  export type ParentPermissionState = PermissionState | 'per_tool';

  interface SubToolItem {
    id: string;
    name: string;
    permission: PermissionState;
  }

  interface ExtensionItem {
    id: string;
    name: string;
    icon: string;
    section: 'Built-In Tools' | 'Apps' | 'Skills' | 'Recipes';
    permission: ParentPermissionState;
    tools?: SubToolItem[];
  }

  let extensions = $state<ExtensionItem[]>(loadExtensions());
  let selectedExtForTools = $state<ExtensionItem | null>(null);

  function loadFavorites(): string[] {
    try {
      const stored = localStorage.getItem('hirn_favorite_models');
      return stored ? JSON.parse(stored) : ['local/gemma-4-9b-it', 'anthropic/claude-3-5-sonnet'];
    } catch {
      return ['local/gemma-4-9b-it', 'anthropic/claude-3-5-sonnet'];
    }
  }

  function loadExtensions(): ExtensionItem[] {
    const defaults: ExtensionItem[] = [
      // Apps Section
      { id: 'ext-apps', name: 'Apps', icon: '📦', section: 'Apps', permission: 'allow', tools: [{ id: 'app_sandbox', name: 'App Sandbox', permission: 'allow' }] },
      { id: 'ext-dev', name: 'Developer', icon: '⚡', section: 'Apps', permission: 'per_tool', tools: [{ id: 'read_file', name: 'Read File', permission: 'allow' }, { id: 'write_file', name: 'Write File', permission: 'ask' }, { id: 'run_cmd', name: 'Run Command', permission: 'ask' }] },
      { id: 'ext-mgr', name: 'Extension Manager', icon: '🧩', section: 'Apps', permission: 'allow' },
      { id: 'ext-todo', name: 'Todo', icon: '☑️', section: 'Apps', permission: 'allow', tools: [{ id: 'manage_task', name: 'Manage Task', permission: 'allow' }] },
      { id: 'ext-visualiser', name: 'Autovisualiser', icon: '🎨', section: 'Apps', permission: 'allow', tools: [{ id: 'generate_image', name: 'Generate Image', permission: 'allow' }] },

      // Skills Section
      { id: 'ext-analyze', name: 'Analyze', icon: '🔍', section: 'Skills', permission: 'allow', tools: [{ id: 'grep_search', name: 'Grep Search', permission: 'allow' }, { id: 'code_survey', name: 'Code Survey', permission: 'allow' }] },
      { id: 'ext-skills', name: 'Skills', icon: '🎯', section: 'Skills', permission: 'allow', tools: [{ id: 'linear_mcp', name: 'Linear MCP', permission: 'allow' }] },

      // Recipes Section
      { id: 'ext-summon', name: 'Summon', icon: '🔮', section: 'Recipes', permission: 'ask', tools: [{ id: 'subagent', name: 'Subagent Dispatch', permission: 'ask' }] },
      { id: 'ext-tom', name: 'Tom', icon: '👤', section: 'Recipes', permission: 'allow' },
      { id: 'ext-tutorial', name: 'Tutorial', icon: '📚', section: 'Recipes', permission: 'allow' },

      // Built-In Tools Section
      { id: 'ext-builtin-read', name: 'Read File', icon: '📄', section: 'Built-In Tools', permission: 'allow' },
      { id: 'ext-builtin-write', name: 'Write File', icon: '✏️', section: 'Built-In Tools', permission: 'ask' },
      { id: 'ext-builtin-cmd', name: 'Run Command', icon: '🐚', section: 'Built-In Tools', permission: 'ask' },
      { id: 'ext-builtin-search', name: 'Web Search', icon: '🌐', section: 'Built-In Tools', permission: 'allow' },
      { id: 'ext-builtin-agent', name: 'Subagent Manager', icon: '🤖', section: 'Built-In Tools', permission: 'ask' }
    ];

    try {
      const stored = localStorage.getItem('hirn_session_extensions_v2');
      if (stored) {
        const parsed = JSON.parse(stored);
        if (Array.isArray(parsed) && parsed.length > 0 && parsed.every(item => item.permission)) {
          return parsed;
        }
      }
    } catch (e) {
      console.warn('Failed to load extensions from localStorage', e);
    }
    return defaults;
  }

  function saveExtensions() {
    try {
      localStorage.setItem('hirn_session_extensions_v2', JSON.stringify(extensions));
    } catch (e) {
      console.warn('Failed to save extensions to localStorage', e);
    }
  }

  function setParentPermission(extId: string, perm: ParentPermissionState) {
    const ext = extensions.find(e => e.id === extId);
    if (!ext) return;

    ext.permission = perm;

    if (perm !== 'per_tool' && ext.tools) {
      ext.tools.forEach(t => t.permission = perm as PermissionState);
    } else if (perm === 'per_tool' && ext.tools && ext.tools.length > 0) {
      selectedExtForTools = ext;
    }

    saveExtensions();
  }

  function setToolPermission(extId: string, toolId: string, perm: PermissionState) {
    const ext = extensions.find(e => e.id === extId);
    if (!ext || !ext.tools) return;

    const tool = ext.tools.find(t => t.id === toolId);
    if (!tool) return;

    tool.permission = perm;

    const allSame = ext.tools.every(t => t.permission === tool.permission);
    if (allSame) {
      ext.permission = tool.permission;
    } else {
      ext.permission = 'per_tool';
    }

    saveExtensions();
  }

  function handleExtensionRowClick(ext: ExtensionItem) {
    if (ext.tools && ext.tools.length > 0) {
      selectedExtForTools = selectedExtForTools?.id === ext.id ? null : ext;
    }
  }

  // Unified Attachment Menu Triggers
  function toggleAttachMenu(e: MouseEvent) {
    e.stopPropagation();
    isAttachMenuOpen = !isAttachMenuOpen;
    isModelMenuOpen = false;
    isToolsMenuOpen = false;
  }

  function handleAttachFiles() {
    isAttachMenuOpen = false;
    fileInputRef?.click();
  }

  function handleAttachFolder() {
    isAttachMenuOpen = false;
    folderInputRef?.click();
  }

  function handleFileInputChange(e: Event) {
    const target = e.target as HTMLInputElement;
    if (target.files) {
      const files = Array.from(target.files).map(f => ({
        name: f.name,
        path: f.name,
        isFolder: false
      }));
      attachedFiles = [...attachedFiles, ...files];
    }
  }

  function handleFolderInputChange(e: Event) {
    const target = e.target as HTMLInputElement;
    if (target.files && target.files.length > 0) {
      const firstFile = target.files[0];
      const folderName = firstFile.webkitRelativePath.split('/')[0] || firstFile.name;
      attachedFiles = [...attachedFiles, {
        name: folderName,
        path: folderName,
        isFolder: true
      }];
    }
  }

  function removeFile(index: number) {
    attachedFiles = attachedFiles.filter((_, i) => i !== index);
  }

  let totalActiveToolsCount = $derived(() => {
    let count = 0;
    for (const ext of extensions) {
      if (ext.tools && ext.tools.length > 0) {
        count += ext.tools.filter(t => t.permission !== 'off').length;
      } else if (ext.permission !== 'off') {
        count += 1;
      }
    }
    return count;
  });

  let filteredExtensions = $derived(() => {
    const query = toolsSearchQuery.trim().toLowerCase();
    if (!query) return extensions;
    return extensions.filter(e =>
      e.name.toLowerCase().includes(query) ||
      (e.tools && e.tools.some(t => t.name.toLowerCase().includes(query)))
    );
  });

  let groupedExtensions = $derived(() => {
    const all = filteredExtensions();
    const sections: { header: string; items: ExtensionItem[] }[] = [
      { header: 'Apps', items: all.filter(e => e.section === 'Apps') },
      { header: 'Skills', items: all.filter(e => e.section === 'Skills') },
      { header: 'Recipes', items: all.filter(e => e.section === 'Recipes') },
      { header: 'Built-In Tools', items: all.filter(e => e.section === 'Built-In Tools') }
    ];
    return sections.filter(s => s.items.length > 0);
  });

  function saveFavorites(favs: string[]) {
    favoriteModelIds = favs;
    try {
      localStorage.setItem('hirn_favorite_models', JSON.stringify(favs));
    } catch (e) {
      console.warn('Failed to save favorites to localStorage', e);
    }
  }

  function selectModel(modelId: string) {
    if (activeSession) {
      activeSession.selectedModelId = modelId;
      try {
        localStorage.setItem('hirn_selected_model', modelId);
      } catch (e) {
        console.warn('Failed to save selected model to localStorage', e);
      }
    }
    isModelMenuOpen = false;
  }

  function toggleFavorite(event: MouseEvent, modelId: string) {
    event.stopPropagation();
    if (favoriteModelIds.includes(modelId)) {
      saveFavorites(favoriteModelIds.filter(id => id !== modelId));
    } else {
      saveFavorites([...favoriteModelIds, modelId]);
    }
  }

  let selectedModel = $derived(
    activeSession?.availableModels.find(m => m.id === activeSession.selectedModelId) ?? activeSession?.availableModels[0]
  );

  let filteredModels = $derived(() => {
    if (!activeSession) return [];
    const query = searchQuery.trim().toLowerCase();
    if (!query) return activeSession.availableModels;
    return activeSession.availableModels.filter(
      m => m.name.toLowerCase().includes(query) || m.provider.toLowerCase().includes(query)
    );
  });

  let groupedModels = $derived(() => {
    const all = filteredModels();
    const favorites = all.filter(m => favoriteModelIds.includes(m.id));
    const regularMap = new Map<string, AcpModel[]>();

    for (const m of all) {
      const list = regularMap.get(m.provider) || [];
      list.push(m);
      regularMap.set(m.provider, list);
    }

    const groups: { header: string; models: AcpModel[] }[] = [];

    if (favorites.length > 0 && !searchQuery) {
      groups.push({ header: 'Favorite', models: favorites });
    }

    for (const [provider, models] of regularMap.entries()) {
      groups.push({ header: provider, models });
    }

    return groups;
  });

  function handleSubmit() {
    if ((!promptText.trim() && attachedFiles.length === 0) || !activeSession) return;
    let messageText = promptText.trim();
    if (attachedFiles.length > 0) {
      const fileListStr = attachedFiles.map(f => f.isFolder ? `[Attached Folder: ${f.name}](${f.path})` : `[Attached File: ${f.name}](${f.path})`).join('\n');
      messageText = messageText ? `${messageText}\n\n${fileListStr}` : fileListStr;
    }
    store.sendMessage(activeSession.id, messageText);
    promptText = '';
    attachedFiles = [];
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault();
      handleSubmit();
    }
  }

  function handleBackdropClick(e: MouseEvent) {
    const target = e.target as HTMLElement;
    if (!target.closest('.model-picker-container')) {
      isModelMenuOpen = false;
    }
    if (!target.closest('.tools-picker-container')) {
      isToolsMenuOpen = false;
    }
    if (!target.closest('.attach-picker-container')) {
      isAttachMenuOpen = false;
    }
  }
</script>

<svelte:window onclick={handleBackdropClick} />

<div class="flex-1 flex flex-col h-screen bg-background text-foreground overflow-x-hidden relative">
  {#if activeSession}
    <!-- Header with centered Model picker (LibreChat Style) -->
    <header class="flex items-center justify-between px-6 py-3 border-b border-border/60 bg-background relative z-40">
      <div class="flex flex-col min-w-0">
        <h2 class="text-sm font-bold text-foreground truncate max-w-[200px] sm:max-w-xs">{activeSession.title}</h2>
        <div class="flex items-center gap-1.5 mt-0.5">
          <span class="text-[10px] text-muted-foreground bg-muted px-1.5 py-0.5 rounded font-mono border border-border/30">{activeSession.agentName}</span>
        </div>
      </div>

      <div class="flex items-center gap-3">
        <span class="text-[10px] text-muted-foreground bg-muted px-2 py-1 rounded border border-border/30 font-medium">
          {activeSession.status}
        </span>
      </div>
    </header>

    <!-- Chat Messages Feed -->
    <div class="flex-1 overflow-y-auto px-4 py-6 md:px-8 space-y-6 scrollbar-thin">
      <div class="max-w-[768px] mx-auto space-y-6">
        {#each activeSession.messages as msg (msg.id)}
          <div class="flex flex-col gap-2">
            {#if msg.role === 'user'}
              <!-- User message: rounded card bubble -->
              <div class="flex justify-end">
                <div class="bg-muted rounded-2xl px-4 py-2.5 text-sm text-foreground max-w-[85%] whitespace-pre-wrap">
                  {msg.content}
                </div>
              </div>
            {:else}
              <!-- Assistant message: clean markdown structure with details for thoughts -->
              <div class="flex flex-col gap-3">
                {#if msg.thoughts}
                  <details class="bg-muted/40 rounded-xl p-3 text-xs text-muted-foreground group">
                    <summary class="font-semibold text-foreground/80 cursor-pointer list-none flex items-center justify-between">
                      <span>Thought Process</span>
                      <ChevronDown class="size-3.5 text-gray-500 transition-transform group-open:rotate-180" />
                    </summary>
                    <p class="mt-2 leading-relaxed whitespace-pre-wrap pt-2">{msg.thoughts}</p>
                  </details>
                {/if}

                {#if msg.toolCalls}
                  <div class="space-y-2">
                    {#each msg.toolCalls as tool}
                      <ToolExecutionCard toolCall={tool} />
                    {/each}
                  </div>
                {/if}

                {#if msg.content}
                  <div class="text-sm text-foreground/90 leading-relaxed whitespace-pre-wrap select-text">
                    {msg.content}
                  </div>
                {/if}
              </div>
            {/if}
          </div>
        {:else}
          <div class="h-[40vh] flex flex-col items-center justify-center text-center gap-4 px-4">
            <div class="size-12 rounded-full bg-gradient-to-tr from-indigo-500 to-purple-600 flex items-center justify-center text-lg font-black text-white shadow-lg animate-pulse">
              H
            </div>
            <div class="space-y-1">
              <h3 class="text-base font-bold text-foreground">How can I help you today?</h3>
              <p class="text-xs text-muted-foreground max-w-sm">Select a model at the top, configure your active tools, or ask questions to get started.</p>
            </div>
          </div>
        {/each}
      </div>
    </div>

    <!-- Centered Floating Input Bar (LibreChat Style) -->
    <div class="p-4 md:pb-6 bg-background">
      <div class="max-w-[768px] mx-auto w-full relative">
        <InputGroup.Root class="!opacity-100 !bg-muted/50 border !border-transparent focus-within:!border-border rounded-[24px] shadow-md focus-within:shadow-lg transition-all p-3 flex flex-col gap-2.5">
          <!-- Attached Pills -->
          {#if attachedFiles.length > 0}
            <div class="flex flex-wrap gap-1.5 px-2">
              {#each attachedFiles as file, idx (idx)}
                <div class="flex items-center gap-1.5 bg-background border border-border rounded-xl px-2.5 py-1 text-xs text-foreground shadow-sm">
                  <span>{file.isFolder ? '📁' : '📎'}</span>
                  <span class="max-w-[150px] truncate font-medium">{file.name}</span>
                  <button onclick={() => removeFile(idx)} class="text-gray-500 hover:text-white transition-colors">
                    <X class="size-3.5" />
                  </button>
                </div>
              {/each}
            </div>
          {/if}

          <!-- Textarea prompt -->
          <InputGroup.Textarea
            placeholder="Ask, Search or Chat..."
            bind:value={promptText}
            onkeydown={handleKeyDown}
            rows={2}
            class="w-full !bg-transparent border-none outline-none text-sm text-foreground py-1 px-3 resize-none leading-relaxed min-h-[60px]"
          />

          <!-- Input bar tools -->
          <InputGroup.Addon align="block-end" class="flex items-center border-t border-border/40 pt-2 px-1">
            <!-- Left aligned controls -->
            <div class="flex items-center gap-3">
              <!-- Model Selection Trigger inside Prompt bottom bar -->
              <div class="relative model-picker-container">
                <DropdownMenu.Root bind:open={isModelMenuOpen}>
                  <DropdownMenu.Trigger>
                    {#snippet child({ props })}
                      <InputGroup.Button
                        {...props}
                        variant="ghost"
                        class="flex items-center gap-1.5 h-7 px-2.5 bg-muted hover:bg-muted/80 border border-border rounded-xl text-[10px] text-foreground/80 font-medium transition-all"
                      >
                        <span class="size-3.5 rounded bg-background flex items-center justify-center text-[8px] font-mono border border-border shrink-0 text-foreground mr-1">
                          {#if selectedModel?.provider.includes('Google')}G{:else if selectedModel?.provider.includes('Anthropic')}A{:else if selectedModel?.provider.includes('Meta')}M{:else if selectedModel?.provider.includes('LMStudio')}LM{:else}L{/if}
                        </span>
                        <span>{selectedModel?.name || selectedModel?.id}</span>
                        <ChevronDown class="size-3 text-gray-500 ml-1" />
                      </InputGroup.Button>
                    {/snippet}
                  </DropdownMenu.Trigger>
                  <DropdownMenu.Content
                    side="top"
                    align="start"
                    class="w-[320px] bg-popover border border-border text-popover-foreground rounded-xl shadow-2xl z-50 overflow-hidden flex flex-col p-0 animate-in fade-in duration-100"
                  >
                    <div class="p-2 border-b border-border flex items-center gap-2">
                      <Search class="size-4 text-gray-500 shrink-0" />
                      <input
                        type="text"
                        placeholder="Search models..."
                        class="w-full bg-transparent border-none outline-none text-xs text-foreground placeholder-muted-foreground/60 py-1"
                        bind:value={searchQuery}
                        onclick={(e) => e.stopPropagation()}
                      />
                    </div>
                    <div class="max-h-[280px] overflow-y-auto p-1.5 flex flex-col gap-3">
                      {#each groupedModels() as group}
                        <div class="flex flex-col gap-0.5">
                          <div class="text-[9px] font-semibold text-gray-500 uppercase tracking-wider px-2.5 py-1 font-mono">
                            {group.header}
                          </div>
                          {#each group.models as model}
                            <DropdownMenu.Item
                              onclick={() => selectModel(model.id)}
                              class="w-full flex items-center justify-between px-2.5 py-2 rounded-lg text-xs text-left hover:bg-accent hover:text-accent-foreground transition-all cursor-pointer {model.id === activeSession.selectedModelId ? 'bg-accent text-accent-foreground font-medium border border-border' : 'text-foreground/80'}"
                            >
                              <div class="flex items-center gap-2 min-w-0">
                                <span class="size-4 rounded bg-muted flex items-center justify-center text-[9px] font-mono border border-border shrink-0">
                                  {#if model.provider.includes('Google')}G{:else if model.provider.includes('Anthropic')}A{:else if model.provider.includes('Meta')}M{:else if model.provider.includes('LMStudio')}LM{:else}L{/if}
                                </span>
                                <span class="truncate">{model.name}</span>
                              </div>
                              <div class="flex items-center gap-1">
                                <span
                                  onclick={(e) => toggleFavorite(e, model.id)}
                                  onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') toggleFavorite(e, model.id); }}
                                  role="button"
                                  tabindex="0"
                                  class="p-1 hover:text-yellow-400 text-gray-500 transition-colors cursor-pointer"
                                >
                                  <Star class="size-3 {favoriteModelIds.includes(model.id) ? 'fill-yellow-400 text-yellow-400' : ''}" />
                                </span>
                                {#if model.id === activeSession.selectedModelId}
                                  <Check class="size-3.5 text-emerald-400 shrink-0" />
                                {/if}
                              </div>
                            </DropdownMenu.Item>
                          {/each}
                        </div>
                      {/each}
                    </div>
                  </DropdownMenu.Content>
                </DropdownMenu.Root>
              </div>

              <!-- Context limits / token counter badge -->
              <InputGroup.Text class="text-[10px] text-muted-foreground font-mono">6k / 128k</InputGroup.Text>
            </div>

            <!-- Right aligned controls -->
            <div class="flex items-center gap-1.5 ms-auto">
              <!-- Tools Extensions Popover -->
              <div class="relative tools-picker-container">
                <button
                  onclick={(e) => { e.stopPropagation(); isToolsMenuOpen = !isToolsMenuOpen; isModelMenuOpen = false; isAttachMenuOpen = false; }}
                  class="flex items-center gap-1.5 px-2.5 py-1 bg-muted hover:bg-muted/80 border border-border rounded-xl text-[10px] font-mono text-foreground/80 transition-all cursor-pointer h-7"
                  title="Session Extensions & Tool Permissions"
                >
                  <Wrench class="size-3 text-gray-400" />
                  <span>{totalActiveToolsCount()}</span>
                </button>

                {#if isToolsMenuOpen}
                  <div class="absolute bottom-full right-0 mb-2 w-[320px] bg-popover border border-border text-popover-foreground rounded-xl shadow-2xl z-50 overflow-hidden flex flex-col animate-in fade-in duration-150">
                    <div class="p-2 border-b border-border flex items-center gap-2">
                      <Search class="size-4 text-gray-500 shrink-0" />
                      <input
                        type="text"
                        placeholder="Search tools..."
                        class="w-full bg-transparent border-none outline-none text-xs text-foreground placeholder-muted-foreground/60 py-1"
                        bind:value={toolsSearchQuery}
                        onclick={(e) => e.stopPropagation()}
                      />
                    </div>
                    <div class="max-h-[280px] overflow-y-auto p-1.5 flex flex-col gap-3">
                      {#each groupedExtensions() as group}
                        <div class="flex flex-col gap-0.5">
                          <div class="text-[9px] font-semibold text-gray-500 uppercase tracking-wider px-2.5 py-1 font-mono">
                            {group.header}
                          </div>
                          {#each group.items as ext (ext.id)}
                            <div
                              class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-xs text-left hover:bg-accent hover:text-accent-foreground transition-all cursor-pointer"
                              role="button"
                              tabindex="0"
                              onclick={() => handleExtensionRowClick(ext)}
                              onkeydown={(e) => { if (e.key === 'Enter') handleExtensionRowClick(ext); }}
                            >
                              <div class="flex items-center gap-2 min-w-0">
                                <span class="text-sm shrink-0">{ext.icon}</span>
                                <span class="truncate text-foreground font-medium">{ext.name}</span>
                              </div>
                              <select
                                class="bg-muted border border-border text-[10px] text-foreground px-2 py-0.5 rounded cursor-pointer outline-none"
                                value={ext.permission}
                                onclick={(e) => e.stopPropagation()}
                                onchange={(e) => setParentPermission(ext.id, (e.target as HTMLSelectElement).value as ParentPermissionState)}
                              >
                                {#if ext.tools && ext.tools.length > 0}
                                  <option value="per_tool">Per Tool</option>
                                {/if}
                                <option value="off">Off</option>
                                <option value="ask">Ask</option>
                                <option value="allow">Allow</option>
                              </select>
                            </div>

                            {#if selectedExtForTools?.id === ext.id && ext.tools}
                              <div class="pl-6 flex flex-col gap-0.5 border-l border-border ml-4 mt-0.5 mb-1.5">
                                {#each ext.tools as tool (tool.id)}
                                  <div class="flex items-center justify-between py-1 px-2 hover:bg-accent hover:text-accent-foreground rounded">
                                    <span class="text-[11px] text-muted-foreground truncate">{tool.name}</span>
                                    {#if ext.permission === 'per_tool'}
                                      <select
                                        class="bg-muted border border-border text-[9px] text-foreground px-1.5 py-0.5 rounded cursor-pointer outline-none"
                                        value={tool.permission}
                                        onclick={(e) => e.stopPropagation()}
                                        onchange={(e) => setToolPermission(ext.id, tool.id, (e.target as HTMLSelectElement).value as PermissionState)}
                                      >
                                        <option value="off">Off</option>
                                        <option value="ask">Ask</option>
                                        <option value="allow">Allow</option>
                                      </select>
                                    {/if}
                                  </div>
                                {/each}
                              </div>
                            {/if}
                          {/each}
                        </div>
                      {/each}
                    </div>
                  </div>
                {/if}
              </div>

              <!-- Attach Popover -->
              <div class="relative attach-picker-container">
                <InputGroup.Button
                  variant="ghost"
                  class="rounded-full size-7 flex items-center justify-center p-0 text-muted-foreground hover:text-foreground hover:bg-muted bg-transparent border-none"
                  onclick={toggleAttachMenu}
                  title="Attach files or folders"
                >
                  <Paperclip class="size-4" />
                </InputGroup.Button>

                {#if isAttachMenuOpen}
                  <div class="absolute bottom-full right-0 mb-2 w-[160px] bg-popover border border-border rounded-xl shadow-2xl z-50 p-1 flex flex-col gap-0.5 animate-in fade-in duration-100">
                    <button onclick={handleAttachFiles} class="flex items-center gap-2.5 w-full px-3 py-2 text-xs rounded-lg hover:bg-accent hover:text-accent-foreground text-foreground text-left transition-all">
                      <File class="size-3.5 text-muted-foreground" />
                      <span>Attach Files...</span>
                    </button>
                    <button onclick={handleAttachFolder} class="flex items-center gap-2.5 w-full px-3 py-2 text-xs rounded-lg hover:bg-accent hover:text-accent-foreground text-foreground text-left transition-all">
                      <Folder class="size-3.5 text-muted-foreground" />
                      <span>Attach Folder...</span>
                    </button>
                  </div>
                {/if}
              </div>

              <Separator orientation="vertical" class="!h-4 bg-border" />

              <!-- Send button -->
              <InputGroup.Button
                variant="default"
                class="rounded-full size-7 flex items-center justify-center p-0"
                disabled={!promptText.trim() && attachedFiles.length === 0}
                onclick={handleSubmit}
                title="Send Message"
              >
                <ArrowUpIcon class="size-4" />
                <span class="sr-only">Send</span>
              </InputGroup.Button>
            </div>
          </InputGroup.Addon>
        </InputGroup.Root>
      </div>
    </div>

    <!-- Hidden HTML input elements -->
    <input
      type="file"
      multiple
      bind:this={fileInputRef}
      onchange={handleFileInputChange}
      style="display: none;"
    />
    <input
      type="file"
      webkitdirectory
      bind:this={folderInputRef}
      onchange={handleFolderInputChange}
      style="display: none;"
    />
  {:else}
    <div class="flex-1 flex flex-col items-center justify-center text-center gap-3">
      <Compass class="size-10 text-gray-500 animate-spin" />
      <p class="text-sm text-gray-400">No active session selected. Create or select a session to begin.</p>
    </div>
  {/if}
</div>

<style>
  /* Custom scrollbar styling */
  .scrollbar-thin::-webkit-scrollbar {
    width: 6px;
    height: 6px;
  }
  .scrollbar-thin::-webkit-scrollbar-track {
    background: transparent;
  }
  .scrollbar-thin::-webkit-scrollbar-thumb {
    background: #2f2f2f;
    border-radius: 4px;
  }
  .scrollbar-thin::-webkit-scrollbar-thumb:hover {
    background: #3f3f3f;
  }
</style>
