<script lang="ts">
  import type { SessionStore } from '../stores/session.svelte';
  import { 
    MessageSquare, 
    Plus, 
    PanelLeftClose, 
    PanelLeft, 
    Settings, 
    Cpu, 
    Bot, 
    Boxes, 
    User,
    Ellipsis
  } from '@lucide/svelte';

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
  let isProfileMenuOpen = $state(false);

  function handleNewSession() {
    activeNav = 'sessions';
    store.createSession(store.agents[0].id);
  }

  // Simple reactive helper to group sessions
  const groupedSessions = $derived.by(() => {
    const today: typeof store.sessions = [];
    const yesterday: typeof store.sessions = [];
    const older: typeof store.sessions = [];

    const now = Date.now();
    const oneDay = 24 * 60 * 60 * 1000;

    store.sessions.forEach(session => {
      const diff = now - (session.updatedAt || session.createdAt || now);
      if (diff < oneDay) {
        today.push(session);
      } else if (diff < 2 * oneDay) {
        yesterday.push(session);
      } else {
        older.push(session);
      }
    });

    return [
      { label: 'Today', items: today },
      { label: 'Yesterday', items: yesterday },
      { label: 'Previous 7 Days', items: older }
    ].filter(group => group.items.length > 0);
  });
</script>

<aside class="flex flex-col bg-sidebar h-screen transition-all duration-300 select-none border-r border-sidebar-border text-sidebar-foreground {collapsed ? 'w-[60px]' : 'w-[260px]'}">
  <!-- Top header bar -->
  <div class="flex items-center justify-between p-3 gap-2 border-b border-sidebar-border/40">
    {#if !collapsed}
      <div class="flex-1 min-w-0">
        <select 
          bind:value={activeWorkspaceId} 
          class="w-full bg-sidebar border border-sidebar-border text-sidebar-foreground px-2 py-1.5 rounded-lg text-xs font-semibold cursor-pointer outline-none truncate hover:bg-sidebar-accent transition-all"
        >
          {#each workspaces as ws}
            <option value={ws.id} class="bg-sidebar text-sidebar-foreground">{ws.name}</option>
          {/each}
        </select>
      </div>
    {/if}
    
    <button 
      class="p-2 text-sidebar-foreground/60 hover:text-sidebar-foreground hover:bg-sidebar-accent rounded-lg transition-all"
      onclick={() => collapsed = !collapsed}
      title={collapsed ? "Expand Sidebar" : "Collapse Sidebar"}
    >
      {#if collapsed}
        <PanelLeft class="size-4" />
      {:else}
        <PanelLeftClose class="size-4" />
      {/if}
    </button>
  </div>

  <!-- New Chat button -->
  <div class="p-3">
    {#if collapsed}
      <button 
        onclick={handleNewSession}
        class="w-full flex items-center justify-center p-2.5 bg-sidebar hover:bg-sidebar-accent border border-sidebar-border text-sidebar-foreground rounded-lg transition-all"
        title="New"
      >
        <Plus class="size-4" />
      </button>
    {:else}
      <button 
        onclick={handleNewSession}
        class="w-full flex items-center justify-between px-3 py-2 bg-sidebar hover:bg-sidebar-accent border border-sidebar-border text-sidebar-foreground rounded-lg transition-all text-sm font-medium"
      >
        <span>New</span>
        <Plus class="size-4 text-sidebar-foreground/60" />
      </button>
    {/if}
  </div>

  <!-- Session List area -->
  <div class="flex-1 overflow-y-auto px-3 py-2 scrollbar-thin">
    {#if collapsed}
      <div class="flex flex-col items-center gap-2">
        {#each store.sessions as session}
          <button
            onclick={() => {
              activeNav = 'sessions';
              store.selectSession(session.id);
            }}
            class="flex items-center justify-center size-9 rounded-lg transition-all relative {activeNav === 'sessions' && store.activeSessionId === session.id ? 'bg-sidebar-accent text-sidebar-accent-foreground' : 'hover:bg-sidebar-accent/50 text-sidebar-foreground/70'}"
            title={session.title}
          >
            <div class="relative">
              <MessageSquare class="size-4" />
              <span class="absolute -top-0.5 -right-0.5 size-1.5 rounded-full border border-sidebar {session.status === 'working' ? 'bg-blue-500' : session.status === 'finished' ? 'bg-emerald-500' : session.status === 'needs_input' ? 'bg-amber-400' : 'bg-gray-600'}" title={session.status}></span>
            </div>
          </button>
        {/each}
      </div>
    {:else}
      <div class="flex flex-col gap-5">
        {#each groupedSessions as group}
          <div class="flex flex-col gap-1">
            <div class="text-[11px] font-semibold text-sidebar-foreground/50 uppercase tracking-wider px-2 py-1">
              {group.label}
            </div>
            <div class="flex flex-col gap-0.5">
              {#each group.items as session}
                <button
                  onclick={() => {
                    activeNav = 'sessions';
                    store.selectSession(session.id);
                  }}
                  class="group flex items-center gap-3 px-3 py-2 rounded-lg text-sm text-left transition-all {activeNav === 'sessions' && store.activeSessionId === session.id ? 'bg-sidebar-accent text-sidebar-accent-foreground font-medium' : 'hover:bg-sidebar-accent/50 text-sidebar-foreground/80'}"
                >
                  <div class="relative shrink-0">
                    <MessageSquare class="size-4 text-sidebar-foreground/60 group-hover:text-sidebar-foreground transition-colors" />
                    <span class="absolute -top-0.5 -right-0.5 size-1.5 rounded-full border border-sidebar {session.status === 'working' ? 'bg-blue-500' : session.status === 'finished' ? 'bg-emerald-500' : session.status === 'needs_input' ? 'bg-amber-400' : 'bg-gray-600'}" title={session.status}></span>
                  </div>
                  <span class="truncate flex-1">{session.title}</span>
                </button>
              {/each}
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Bottom Navigation Row -->
  <div class="flex flex-col gap-0.5 p-2 border-t border-sidebar-border/40 bg-sidebar">
    <button
      onclick={() => activeNav = 'apps'}
      class="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-xs transition-all {activeNav === 'apps' ? 'bg-sidebar-accent text-sidebar-accent-foreground font-medium' : 'hover:bg-sidebar-accent/50 text-sidebar-foreground/70'}"
      title="Apps, Tools & Skills"
    >
      <Boxes class="size-4 shrink-0" />
      {#if !collapsed}<span class="truncate text-left flex-1">Apps, Tools & Skills</span>{/if}
    </button>

    <button
      onclick={() => activeNav = 'agents'}
      class="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-xs transition-all {activeNav === 'agents' ? 'bg-sidebar-accent text-sidebar-accent-foreground font-medium' : 'hover:bg-sidebar-accent/50 text-sidebar-foreground/70'}"
      title="Agents & Providers"
    >
      <Bot class="size-4 shrink-0" />
      {#if !collapsed}<span class="truncate text-left flex-1">Agents & Providers</span>{/if}
    </button>

    <button
      onclick={() => activeNav = 'models'}
      class="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-xs transition-all {activeNav === 'models' ? 'bg-sidebar-accent text-sidebar-accent-foreground font-medium' : 'hover:bg-sidebar-accent/50 text-sidebar-foreground/70'}"
      title="Model Configuration"
    >
      <Cpu class="size-4 shrink-0" />
      {#if !collapsed}<span class="truncate text-left flex-1">Model Configuration</span>{/if}
    </button>

    <button
      onclick={() => activeNav = 'settings'}
      class="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-xs transition-all {activeNav === 'settings' ? 'bg-sidebar-accent text-sidebar-accent-foreground font-medium' : 'hover:bg-sidebar-accent/50 text-sidebar-foreground/70'}"
      title="Settings"
    >
      <Settings class="size-4 shrink-0" />
      {#if !collapsed}<span class="truncate text-left flex-1">Settings</span>{/if}
    </button>
  </div>
</aside>

<style>
  /* Custom scrollbar styling */
  .scrollbar-thin::-webkit-scrollbar {
    width: 4px;
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
