<script lang="ts">
  import type { SessionStore } from '../stores/session.svelte';

  let { store, activeNav = $bindable('sessions') }: {
    store: SessionStore;
    activeNav: 'sessions' | 'apps' | 'agents' | 'models' | 'settings';
  } = $props();

  let collapsed = $state(false);
  let workspaces = $state([
    { id: 'ws-1', name: 'hirnlabs/hirn' },
    { id: 'ws-2', name: 'Personal Projects' },
    { id: 'ws-3', name: 'Research Lab' }
  ]);
  let activeWorkspaceId = $state('ws-1');

  function handleNewSession() {
    activeNav = 'sessions';
    store.createSession(store.agents[0].id);
  }
</script>

<aside class="sidebar {collapsed ? 'collapsed' : ''}">
  <div class="sidebar-top">
    <div class="header">
      <button class="collapse-btn" onclick={() => collapsed = !collapsed} title={collapsed ? "Expand Sidebar" : "Collapse Sidebar"}>
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
          {#if collapsed}
            <polyline points="9 18 15 12 9 6"/>
          {:else}
            <polyline points="15 18 9 12 15 6"/>
          {/if}
        </svg>
      </button>

      {#if !collapsed}
        <div class="workspace-selector">
          <select bind:value={activeWorkspaceId} class="workspace-select" title="Active Workspace">
            {#each workspaces as ws}
              <option value={ws.id}>{ws.name}</option>
            {/each}
          </select>
        </div>
        <button class="new-session-btn" onclick={handleNewSession} title="New Session">+</button>
      {/if}
    </div>

    {#if !collapsed}
      <div class="sessions-section">
        <div class="section-label">Sessions</div>
        <div class="session-list">
          {#each store.sessions as session}
            <button
              class="session-item {activeNav === 'sessions' && store.activeSessionId === session.id ? 'active' : ''}"
              onclick={() => {
                activeNav = 'sessions';
                store.selectSession(session.id);
              }}
            >
              <span class="status-dot {session.status}">●</span>
              <span class="session-title">{session.title}</span>
            </button>
          {/each}
        </div>
      </div>
    {:else}
      <div class="collapsed-sessions">
        <button class="icon-btn new-icon" onclick={handleNewSession} title="New Session">+</button>
        {#each store.sessions as session}
          <button
            class="icon-btn {activeNav === 'sessions' && store.activeSessionId === session.id ? 'active' : ''}"
            onclick={() => {
              activeNav = 'sessions';
              store.selectSession(session.id);
            }}
            title="{session.title} ({session.status})"
          >
            <span class="status-dot {session.status}">●</span>
          </button>
        {/each}
      </div>
    {/if}
  </div>

  <div class="sidebar-bottom-nav">
    <button
      class="nav-item {activeNav === 'apps' ? 'active' : ''}"
      onclick={() => activeNav = 'apps'}
      title="Apps & Tools"
    >
      <span class="nav-icon">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="3" y="3" width="7" height="7" rx="1"/>
          <rect x="14" y="3" width="7" height="7" rx="1"/>
          <rect x="14" y="14" width="7" height="7" rx="1"/>
          <rect x="3" y="14" width="7" height="7" rx="1"/>
        </svg>
      </span>
      {#if !collapsed}<span class="nav-label">Apps</span>{/if}
    </button>

    <button
      class="nav-item {activeNav === 'agents' ? 'active' : ''}"
      onclick={() => activeNav = 'agents'}
      title="Agents"
    >
      <span class="nav-icon">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="4 17 10 11 4 5"/>
          <line x1="12" y1="19" x2="20" y2="19"/>
        </svg>
      </span>
      {#if !collapsed}<span class="nav-label">Agents</span>{/if}
    </button>

    <button
      class="nav-item {activeNav === 'models' ? 'active' : ''}"
      onclick={() => activeNav = 'models'}
      title="Local Models"
    >
      <span class="nav-icon">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="4" y="4" width="16" height="16" rx="2"/>
          <rect x="9" y="9" width="6" height="6"/>
          <line x1="9" y1="1" x2="9" y2="4"/>
          <line x1="15" y1="1" x2="15" y2="4"/>
          <line x1="20" y1="9" x2="23" y2="9"/>
          <line x1="20" y1="15" x2="23" y2="15"/>
          <line x1="1" y1="9" x2="4" y2="9"/>
          <line x1="1" y1="15" x2="4" y2="15"/>
        </svg>
      </span>
      {#if !collapsed}<span class="nav-label">Models</span>{/if}
    </button>

    <button
      class="nav-item {activeNav === 'settings' ? 'active' : ''}"
      onclick={() => activeNav = 'settings'}
      title="Settings"
    >
      <span class="nav-icon">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="3"/>
          <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/>
        </svg>
      </span>
      {#if !collapsed}<span class="nav-label">Settings</span>{/if}
    </button>
  </div>
</aside>

<style>
  .sidebar {
    width: 240px;
    height: 100vh;
    background: var(--bg-sidebar, #09090b);
    border-right: 1px solid var(--border-color, #18181b);
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    transition: width 0.2s cubic-bezier(0.16, 1, 0.3, 1);
    user-select: none;
  }
  .sidebar.collapsed {
    width: 48px;
  }
  .sidebar-top {
    display: flex;
    flex-direction: column;
    flex: 1;
    overflow: hidden;
  }
  .header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 12px;
    border-bottom: 1px solid var(--border-color, #18181b);
  }
  .collapse-btn {
    background: transparent;
    border: none;
    color: var(--text-muted, #71717a);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 4px;
    border-radius: 4px;
    flex-shrink: 0;
  }
  .collapse-btn:hover { color: var(--text-main, #f4f4f5); background: var(--bg-card, #18181b); }
  .workspace-selector {
    flex: 1;
    overflow: hidden;
  }
  .workspace-select {
    width: 100%;
    background: var(--bg-card, #141417);
    border: 1px solid var(--border-color, #27272a);
    color: var(--text-main, #f4f4f5);
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 11px;
    font-weight: 600;
    cursor: pointer;
    outline: none;
    white-space: nowrap;
    text-overflow: ellipsis;
  }
  .new-session-btn {
    background: var(--bg-card, #18181b);
    border: 1px solid var(--border-color, #27272a);
    color: var(--text-main, #f4f4f5);
    width: 22px;
    height: 22px;
    border-radius: 4px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 14px;
    flex-shrink: 0;
  }
  .new-session-btn:hover { background: var(--border-color, #27272a); }
  .sessions-section {
    flex: 1;
    overflow-y: auto;
    padding: 10px 8px;
  }
  .section-label {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.8px;
    color: var(--text-muted, #52525b);
    padding: 4px 8px;
    font-weight: 600;
  }
  .session-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
    margin-top: 4px;
  }
  .session-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    background: transparent;
    border: none;
    border-radius: 4px;
    color: var(--text-muted, #a1a1aa);
    cursor: pointer;
    font-size: 12px;
    text-align: left;
  }
  .session-item:hover { background: var(--bg-card, #18181b); color: var(--text-main, #f4f4f5); }
  .session-item.active { background: var(--border-color, #27272a); color: var(--text-main, #ffffff); font-weight: 500; }
  .session-title { flex: 1; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .status-dot { font-size: 8px; }
  .status-dot.idle { color: #52525b; }
  .status-dot.working { color: #ffffff; display: inline-block; animation: pulse 1s infinite; }
  .status-dot.waiting_for_input { color: #fbbf24; }
  .status-dot.error { color: #f87171; }

  .collapsed-sessions {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    padding: 10px 0;
    overflow-y: auto;
  }
  .icon-btn {
    width: 32px;
    height: 32px;
    background: transparent;
    border: none;
    border-radius: 4px;
    color: var(--text-muted, #a1a1aa);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .icon-btn:hover { background: var(--bg-card, #18181b); color: var(--text-main, #f4f4f5); }
  .icon-btn.active { background: var(--border-color, #27272a); color: var(--text-main, #ffffff); }
  .new-icon { background: var(--bg-card, #18181b); border: 1px solid var(--border-color, #27272a); }

  .sidebar-bottom-nav {
    border-top: 1px solid var(--border-color, #18181b);
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    background: var(--bg-sidebar, #09090b);
  }
  .nav-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 10px;
    background: transparent;
    border: none;
    border-radius: 4px;
    color: var(--text-muted, #71717a);
    cursor: pointer;
    font-size: 12px;
    transition: color 0.15s, background 0.15s;
  }
  .nav-item:hover { background: var(--bg-card, #18181b); color: var(--text-main, #e4e4e7); }
  .nav-item.active { background: var(--bg-card, #18181b); color: var(--text-main, #ffffff); font-weight: 500; }
  .nav-icon { display: flex; align-items: center; justify-content: center; width: 16px; height: 16px; }
  .nav-label { flex: 1; text-align: left; }
  @keyframes pulse { 0% { opacity: 0.3; } 50% { opacity: 1; } 100% { opacity: 0.3; } }
</style>
