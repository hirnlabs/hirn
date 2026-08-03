<script lang="ts">
  import type { SessionStore } from '../stores/session.svelte';
  import type { AcpModel } from '../types/acp';
  import ToolExecutionCard from './ToolExecutionCard.svelte';

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

<div class="chat-canvas">
  {#if activeSession}
    <header class="chat-header">
      <div class="header-info">
        <h2 class="title">{activeSession.title}</h2>
        <span class="agent-badge">{activeSession.agentName}</span>
      </div>
      <div class="header-status {activeSession.status}">
        Status: {activeSession.status}
      </div>
    </header>

    <!-- Ultra-Minimal Minimalist Chat Feed (Matching Provided Screenshot 1:1) -->
    <div class="messages-container">
      {#each activeSession.messages as msg (msg.id)}
        <div class="message-turn {msg.role}">
          {#if msg.role === 'user'}
            <div class="user-card-bubble">
              {msg.content}
            </div>
          {:else}
            {#if msg.thoughts}
              <details class="thoughts-block">
                <summary>Thought Process</summary>
                <p>{msg.thoughts}</p>
              </details>
            {/if}

            {#if msg.toolCalls}
              {#each msg.toolCalls as tool}
                <ToolExecutionCard toolCall={tool} />
              {/each}
            {/if}

            {#if msg.content}
              <div class="assistant-text-block">
                {msg.content}
              </div>
            {/if}
          {/if}
        </div>
      {/each}
    </div>

    <!-- Goose & 1:1 Zed Floating Prompt Container -->
    <div class="input-container">
      <div class="goose-prompt-box">
        <!-- Attached Files & Folders Display Row -->
        {#if attachedFiles.length > 0}
          <div class="attached-files-row">
            {#each attachedFiles as file, idx (idx)}
              <div class="attached-file-pill">
                <span class="file-icon">{file.isFolder ? '📁' : '📎'}</span>
                <span class="file-name" title={file.path}>{file.name}</span>
                <button class="remove-file-btn" onclick={() => removeFile(idx)}>×</button>
              </div>
            {/each}
          </div>
        {/if}

        <textarea
          class="prompt-textarea"
          placeholder="Type a message or prompt..."
          bind:value={promptText}
          onkeydown={handleKeyDown}
          rows={2}
        ></textarea>

        <div class="prompt-bottom-bar">
          <div class="left-badges">
            <!-- Plain Monochrome Trigger -->
            <div class="model-picker-container">
              <button
                class="model-picker-trigger-clean"
                onclick={(e) => { e.stopPropagation(); isModelMenuOpen = !isModelMenuOpen; isToolsMenuOpen = false; isAttachMenuOpen = false; }}
                title="Select LLM Model"
              >
                <!-- Monochrome Plain Provider Icon -->
                <span class="provider-icon-badge">
                  {#if selectedModel?.provider.includes('Google')}
                    G
                  {:else if selectedModel?.provider.includes('Anthropic')}
                    A
                  {:else if selectedModel?.provider.includes('Meta')}
                    M
                  {:else if selectedModel?.provider.includes('LMStudio')}
                    LM
                  {:else}
                    L
                  {/if}
                </span>

                <span class="model-clean-label">{selectedModel?.name || selectedModel?.id}</span>
              </button>

              <!-- 1:1 Zed Popover Menu -->
              {#if isModelMenuOpen}
                <div class="zed-model-popover-1to1">
                  <!-- Search Bar at Top -->
                  <div class="popover-search-container">
                    <input
                      type="text"
                      class="popover-search-input"
                      placeholder="Select a model..."
                      bind:value={searchQuery}
                      onclick={(e) => e.stopPropagation()}
                    />
                  </div>

                  <!-- Model Groups & Favorites -->
                  <div class="popover-scroll-area">
                    {#each groupedModels() as group}
                      <div class="provider-group">
                        <div class="provider-header">{group.header}</div>
                        {#each group.models as model}
                          <div
                            class="zed-model-row-1to1 {model.id === activeSession.selectedModelId ? 'active' : ''}"
                            role="button"
                            tabindex="0"
                            onclick={() => selectModel(model.id)}
                            onkeydown={(e) => { if (e.key === 'Enter') selectModel(model.id); }}
                          >
                            <div class="model-row-left">
                              <!-- Plain Monochrome Icon in Row -->
                              <span class="provider-icon-badge">
                                {#if model.provider.includes('Google')}
                                  G
                                {:else if model.provider.includes('Anthropic')}
                                  A
                                {:else if model.provider.includes('Meta')}
                                  M
                                {:else if model.provider.includes('LMStudio')}
                                  LM
                                {:else}
                                  L
                                {/if}
                              </span>

                              <span class="zed-model-title">{model.name}</span>
                            </div>

                            <div class="model-row-actions">
                              <!-- Filled Star Shown Only On Hover (Replacing Tick) -->
                              <span
                                class="star-icon {favoriteModelIds.includes(model.id) ? 'starred' : ''}"
                                role="button"
                                tabindex="0"
                                onclick={(e) => toggleFavorite(e, model.id)}
                                onkeydown={(e) => { if (e.key === 'Enter') toggleFavorite(e, model.id); }}
                                title={favoriteModelIds.includes(model.id) ? 'Remove from Favorites' : 'Add to Favorites'}
                              >★</span>

                              {#if model.id === activeSession.selectedModelId}
                                <span class="checkmark">✓</span>
                              {/if}
                            </div>
                          </div>
                        {/each}
                      </div>
                    {/each}
                  </div>

                  <!-- Bottom Footer: Configure Button -->
                  <div class="popover-footer">
                    <button class="configure-btn">
                      <span>Configure</span>
                      <span class="shortcut">alt-shift-c</span>
                    </button>
                  </div>
                </div>
              {/if}
            </div>
          </div>

          <div class="right-tools">
            <span class="metrics">
              <span class="status-dot-green">●</span> 6k / 128k
            </span>

            <!-- Apps & Tools Extensions Popover Trigger -->
            <div class="tools-picker-container">
              <button
                class="tool-count-btn"
                onclick={(e) => { e.stopPropagation(); isToolsMenuOpen = !isToolsMenuOpen; isModelMenuOpen = false; isAttachMenuOpen = false; }}
                title="Manage Extensions for this Chat Session"
              >
                <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <rect x="3" y="3" width="7" height="7" rx="1"/>
                  <rect x="14" y="3" width="7" height="7" rx="1"/>
                  <rect x="14" y="14" width="7" height="7" rx="1"/>
                  <rect x="3" y="14" width="7" height="7" rx="1"/>
                </svg>
                <span>{totalActiveToolsCount()}</span>
              </button>

              <!-- Permission Control Extensions Popover -->
              {#if isToolsMenuOpen}
                <div class="zed-model-popover-1to1 tools-popover">
                  <!-- Search Bar at Top -->
                  <div class="popover-search-container">
                    <input
                      type="text"
                      class="popover-search-input"
                      placeholder="Erweiterungen suchen..."
                      bind:value={toolsSearchQuery}
                      onclick={(e) => e.stopPropagation()}
                    />
                  </div>

                  <div class="popover-scroll-area">
                    {#each groupedExtensions() as group}
                      <div class="provider-group">
                        <div class="provider-header">{group.header}</div>
                        {#each group.items as ext (ext.id)}
                          <div
                            class="zed-model-row-1to1"
                            role="button"
                            tabindex="0"
                            onclick={() => handleExtensionRowClick(ext)}
                            onkeydown={(e) => { if (e.key === 'Enter') handleExtensionRowClick(ext); }}
                          >
                            <div class="model-row-left">
                              <span class="ext-icon-badge">{ext.icon}</span>
                              <span class="zed-model-title">{ext.name}</span>
                            </div>

                            <!-- Uniform Fixed-Width Parent Permission Selector -->
                            <select
                              class="perm-select"
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

                          <!-- Sub-Tools Panel: Expandable by clicking parent row at any time -->
                          {#if selectedExtForTools?.id === ext.id && ext.tools}
                            <div class="subtools-panel">
                              {#each ext.tools as tool (tool.id)}
                                <div class="zed-model-row-1to1 subtool-row-full">
                                  <span class="subtool-name">{tool.name}</span>

                                  {#if ext.permission === 'per_tool'}
                                    <!-- Uniform Fixed-Width Selector shown ONLY in 'Per Tool' mode -->
                                    <select
                                      class="perm-select"
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

                  <!-- Bottom Footer: Configure Button -->
                  <div class="popover-footer">
                    <button class="configure-btn">
                      <span>Configure</span>
                      <span class="shortcut">alt-shift-e</span>
                    </button>
                  </div>
                </div>
              {/if}
            </div>

            <!-- Single Unified Attachment Button with Popover -->
            <div class="attach-picker-container">
              <button class="icon-tool-btn" onclick={toggleAttachMenu} title="Attach Files or Folders">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M21.44 11.05l-9.19 9.19a6 6 0 0 1-8.49-8.49l9.19-9.19a4 4 0 0 1 5.66 5.66l-9.2 9.19a2 2 0 0 1-2.83-2.83l8.49-8.48"/>
                </svg>
              </button>

              {#if isAttachMenuOpen}
                <div class="attach-popover">
                  <button class="attach-option-btn" onclick={handleAttachFiles}>
                    <span class="option-icon">📎</span>
                    <span>Attach Files...</span>
                  </button>
                  <button class="attach-option-btn" onclick={handleAttachFolder}>
                    <span class="option-icon">📁</span>
                    <span>Attach Folder...</span>
                  </button>
                </div>
              {/if}
            </div>

            <!-- Hidden HTML inputs for files and folders -->
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

            <button class="send-circle-btn" onclick={handleSubmit} title="Send Message (Enter)">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                <line x1="12" y1="19" x2="12" y2="5"/>
                <polyline points="5 12 12 5 19 12"/>
              </svg>
            </button>
          </div>
        </div>
      </div>
    </div>
  {:else}
    <div class="empty-state">
      <p>No active session selected. Create or select a session to begin.</p>
    </div>
  {/if}
</div>

<style>
  .chat-canvas {
    flex: 1;
    min-width: 0;
    height: 100vh;
    display: flex;
    flex-direction: column;
    background: var(--bg-canvas, #09090b);
    color: var(--text-main, #e4e4e7);
    overflow-x: hidden;
    box-sizing: border-box;
  }
  .chat-header {
    padding: 12px 20px;
    border-bottom: 1px solid var(--border-color, #18181b);
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: var(--bg-canvas, #09090b);
    box-sizing: border-box;
  }
  .title {
    font-size: 14px;
    font-weight: 600;
    margin: 0 0 2px 0;
    color: var(--text-main, #f4f4f5);
  }
  .agent-badge {
    font-size: 11px;
    background: var(--bg-card, #18181b);
    color: var(--text-muted, #a1a1aa);
    padding: 2px 6px;
    border-radius: 4px;
    border: 1px solid var(--border-color, #27272a);
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  }
  .header-status {
    font-size: 11px;
    text-transform: capitalize;
    color: var(--text-muted, #71717a);
  }
  .header-status.working { color: var(--text-main, #ffffff); }

  /* Ultra-Minimal Chat Feed (Matching Screenshot 1:1) */
  .messages-container {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 20px 24px;
    display: flex;
    flex-direction: column;
    gap: 16px;
    box-sizing: border-box;
  }
  .message-turn {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 12px;
    box-sizing: border-box;
  }

  /* User Message Dark Card Bubble (Matching Screenshot 1:1) */
  .user-card-bubble {
    background: #141518;
    border: 1px solid #27272a;
    border-radius: 8px;
    padding: 10px 14px;
    color: #e4e4e7;
    font-size: 13.5px;
    line-height: 1.5;
    font-family: inherit;
    box-sizing: border-box;
    width: 100%;
    white-space: pre-wrap;
  }

  /* Assistant Text Block (Matching Screenshot 1:1) */
  .assistant-text-block {
    color: #e4e4e7;
    font-size: 13.5px;
    line-height: 1.6;
    padding: 2px 0;
    white-space: pre-wrap;
  }

  .thoughts-block {
    margin: 4px 0;
    font-size: 11px;
    color: var(--text-muted, #71717a);
    background: var(--bg-card, #121215);
    padding: 6px 10px;
    border-radius: 4px;
    border-left: 2px solid var(--border-color, #3f3f46);
  }
  .thoughts-block summary { cursor: pointer; font-weight: 500; }

  /* Prompt Box Container */
  .input-container {
    padding: 16px 24px 20px 24px;
    background: var(--bg-canvas, #09090b);
    box-sizing: border-box;
    width: 100%;
  }
  .goose-prompt-box {
    background: var(--bg-card, #141417);
    border: 1px solid var(--border-color, #27272a);
    border-radius: 12px;
    padding: 12px 14px;
    display: flex;
    flex-direction: column;
    gap: 10px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.25);
    box-sizing: border-box;
    width: 100%;
  }
  .attached-files-row {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    padding-bottom: 2px;
  }
  .attached-file-pill {
    display: flex;
    align-items: center;
    gap: 6px;
    background: #18191d;
    border: 1px solid #27272a;
    border-radius: 6px;
    padding: 3px 8px;
    font-size: 11.5px;
    color: #e4e4e7;
  }
  .file-name {
    max-width: 180px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .remove-file-btn {
    background: transparent;
    border: none;
    color: #71717a;
    cursor: pointer;
    font-size: 13px;
    line-height: 1;
    padding: 0 2px;
  }
  .remove-file-btn:hover {
    color: #f4f4f5;
  }
  .prompt-textarea {
    width: 100%;
    background: transparent;
    border: none;
    outline: none;
    color: var(--text-main, #f4f4f5);
    font-family: inherit;
    font-size: 13px;
    resize: none;
    line-height: 1.5;
    box-sizing: border-box;
  }
  .prompt-bottom-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-top: 4px;
    box-sizing: border-box;
  }
  .left-badges {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }

  /* Plain Monochrome Trigger */
  .model-picker-container, .tools-picker-container, .attach-picker-container {
    position: relative;
  }

  .attach-popover {
    position: absolute;
    bottom: calc(100% + 8px);
    right: 0;
    width: 145px;
    background: #141518;
    border: 1px solid #27272a;
    border-radius: 6px;
    padding: 4px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    box-shadow: 0 12px 28px rgba(0, 0, 0, 0.95);
    z-index: 200;
    box-sizing: border-box;
  }
  .attach-option-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 6px 8px;
    background: transparent;
    border: none;
    border-radius: 4px;
    color: #f4f4f5;
    font-size: 12px;
    font-family: inherit;
    cursor: pointer;
    text-align: left;
    transition: background 0.1s;
    box-sizing: border-box;
  }
  .attach-option-btn:hover {
    background: #27272a;
  }
  .option-icon {
    font-size: 12px;
  }

  .model-picker-trigger-clean {
    display: flex;
    align-items: center;
    gap: 7px;
    background: transparent;
    border: none;
    color: var(--text-main, #f4f4f5);
    padding: 4px 6px;
    font-size: 12px;
    font-family: inherit;
    font-weight: 500;
    cursor: pointer;
    border-radius: 4px;
    transition: opacity 0.15s;
  }
  .model-picker-trigger-clean:hover {
    opacity: 0.85;
  }
  .model-clean-label {
    color: #f4f4f5;
    font-weight: 600;
  }

  /* Plain Monochrome Provider Icon */
  .provider-icon-badge {
    width: 16px;
    height: 16px;
    border-radius: 3px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 11px;
    font-weight: 700;
    font-family: ui-monospace, SFMono-Regular, monospace;
    color: #e4e4e7;
    background: transparent;
    flex-shrink: 0;
  }

  /* 1:1 Opaque Zed Popover Menu */
  .zed-model-popover-1to1 {
    position: absolute;
    bottom: calc(100% + 10px);
    left: 0;
    width: 320px;
    max-width: 90vw;
    background: #141518;
    border: 1px solid #27272a;
    border-radius: 8px;
    display: flex;
    flex-direction: column;
    box-shadow: 0 16px 36px rgba(0, 0, 0, 0.95);
    z-index: 200;
    overflow: hidden;
    box-sizing: border-box;
  }

  .tools-popover {
    right: 0;
    left: auto;
    width: 310px;
  }

  .ext-icon-badge {
    font-size: 13px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    flex-shrink: 0;
  }

  /* Subtools Panel */
  .subtools-panel {
    display: flex;
    flex-direction: column;
    gap: 2px;
    width: 100%;
    box-sizing: border-box;
    padding: 2px 0;
  }

  .subtool-row-full {
    padding-left: 36px !important;
  }

  .subtool-name {
    font-size: 12px;
    color: #a1a1aa;
  }

  /* Plain Uniform Fixed-Width Permission Dropdown Selector */
  .perm-select {
    width: 92px;
    background: #18191d;
    border: 1px solid #27272a;
    border-radius: 4px;
    color: #f4f4f5;
    font-size: 11px;
    font-family: inherit;
    padding: 3px 6px;
    outline: none;
    cursor: pointer;
    margin-left: auto;
    flex-shrink: 0;
    transition: border-color 0.15s, background 0.15s;
    box-sizing: border-box;
  }
  .perm-select:hover {
    border-color: #3f3f46;
    background: #27272a;
  }
  .perm-select option {
    background: #141518;
    color: #f4f4f5;
    padding: 4px;
  }

  .popover-search-container {
    padding: 10px 12px;
    border-bottom: 1px solid #27272a;
    box-sizing: border-box;
  }
  .popover-search-input {
    width: 100%;
    background: transparent;
    border: none;
    outline: none;
    color: #f4f4f5;
    font-size: 13px;
    font-family: inherit;
    box-sizing: border-box;
  }
  .popover-search-input::placeholder {
    color: #52525b;
  }
  .popover-scroll-area {
    max-height: 280px;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 6px;
    box-sizing: border-box;
  }
  .provider-group {
    margin-bottom: 8px;
  }
  .provider-header {
    font-size: 10.5px;
    font-weight: 500;
    color: #71717a;
    padding: 6px 10px 4px 10px;
    font-family: ui-monospace, SFMono-Regular, monospace;
    text-transform: uppercase;
  }
  .zed-model-row-1to1 {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 7px 10px;
    background: transparent;
    border: none;
    border-radius: 4px;
    color: #f4f4f5;
    font-size: 12.5px;
    cursor: pointer;
    text-align: left;
    transition: background 0.1s;
    outline: none;
    box-sizing: border-box;
  }
  .zed-model-row-1to1:hover {
    background: #27272a;
  }
  .zed-model-row-1to1.active {
    background: #27272a;
  }
  .model-row-left {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
  }
  .zed-model-title {
    color: #f4f4f5;
    font-weight: 400;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .model-row-actions {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }

  /* Star & Hover Replacement Behavior */
  .star-icon {
    font-size: 13px;
    cursor: pointer;
    color: #71717a;
    display: none;
    transition: color 0.1s;
  }
  .star-icon.starred {
    color: #f4f4f5;
  }

  /* On Row Hover: Show Star, Hide Checkmark */
  .zed-model-row-1to1:hover .star-icon {
    display: inline-block;
  }
  .zed-model-row-1to1:hover .checkmark {
    display: none;
  }

  .checkmark {
    color: #38bdf8;
    font-size: 13px;
    font-weight: bold;
  }

  /* Popover Footer */
  .popover-footer {
    padding: 6px;
    border-top: 1px solid #27272a;
    background: #0d0e10;
    box-sizing: border-box;
  }
  .configure-btn {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 6px 10px;
    background: #18191c;
    border: 1px solid #27272a;
    border-radius: 6px;
    color: #e4e4e7;
    font-size: 11px;
    cursor: pointer;
    transition: background 0.15s;
    box-sizing: border-box;
  }
  .configure-btn:hover {
    background: #27272a;
  }
  .shortcut {
    color: #71717a;
    font-size: 10px;
    font-family: ui-monospace, monospace;
  }

  .right-tools {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-shrink: 0;
  }
  .metrics {
    font-size: 11px;
    color: var(--text-muted, #71717a);
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  }
  .status-dot-green {
    color: #10b981;
    font-size: 9px;
  }
  .tool-count {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    color: var(--text-muted, #71717a);
    font-family: ui-monospace, monospace;
  }
  .tool-count-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    color: var(--text-muted, #71717a);
    font-family: ui-monospace, monospace;
    background: transparent;
    border: none;
    cursor: pointer;
    padding: 4px;
    border-radius: 4px;
    transition: color 0.15s;
  }
  .tool-count-btn:hover {
    color: var(--text-main, #f4f4f5);
  }
  .icon-tool-btn {
    background: transparent;
    border: none;
    color: var(--text-muted, #71717a);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 4px;
    border-radius: 4px;
  }
  .icon-tool-btn:hover {
    color: var(--text-main, #f4f4f5);
  }
  .send-circle-btn {
    width: 28px;
    height: 28px;
    border-radius: 50%;
    background: #3f3f46;
    border: none;
    color: #ffffff;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: background 0.15s, transform 0.1s;
    flex-shrink: 0;
  }
  .send-circle-btn:hover {
    background: #52525b;
    transform: scale(1.05);
  }
  .empty-state {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted, #52525b);
    font-size: 13px;
  }
</style>
