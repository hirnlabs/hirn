<script lang="ts">
  import { onMount } from 'svelte';
  import type { SessionStore } from '../stores/session.svelte';
  import { Button } from '$lib/components/ui/button';
  import * as Select from '$lib/components/ui/select';
  import { Badge } from '$lib/components/ui/badge';
  import { 
    MessageSquare, 
    Plus, 
    Mic,
    PanelLeftClose, 
    PanelLeft, 
    Settings, 
    Cpu, 
    Bot, 
    Boxes,
    Archive,
    History
  } from '@lucide/svelte';

  let { store, activeNav = $bindable('sessions') }: {
    store: SessionStore;
    activeNav: 'sessions' | 'apps' | 'agents' | 'models' | 'settings' | 'history';
  } = $props();

  let collapsed = $state(false);
  let preferVoiceInput = $state(false);

  let workspaces = $state([
    { id: 'ws-1', name: 'hirnlabs/hirn' },
    { id: 'ws-2', name: 'Personal Projects' },
    { id: 'ws-3', name: 'Research Lab' }
  ]);
  let activeWorkspaceId = $state('ws-1');

  onMount(() => {
    if (typeof localStorage !== 'undefined') {
      preferVoiceInput = localStorage.getItem('hirn_prefer_voice_input') === 'true';
    }
    const updateVoicePref = () => {
      if (typeof localStorage !== 'undefined') {
        preferVoiceInput = localStorage.getItem('hirn_prefer_voice_input') === 'true';
      }
    };
    window.addEventListener('storage', updateVoicePref);
    return () => window.removeEventListener('storage', updateVoicePref);
  });

  function handleNewSession(startVoice = false) {
    activeNav = 'sessions';
    store.createSession(store.agents[0].id);
    if (startVoice) {
      store.autoStartRecording = true;
    }
  }

  const groupedSessions = $derived.by(() => {
    const today: typeof store.sessions = [];
    const yesterday: typeof store.sessions = [];
    const older: typeof store.sessions = [];

    const now = Date.now();
    const oneDay = 24 * 60 * 60 * 1000;

    const unarchived = store.sessions.filter(s => !s.archived);

    unarchived.forEach(session => {
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

  let activeWorkspaceLabel = $derived(
    workspaces.find(w => w.id === activeWorkspaceId)?.name ?? workspaces[0].name
  );
</script>

<aside class="flex flex-col bg-sidebar h-screen transition-all duration-300 select-none border-r border-sidebar-border text-sidebar-foreground {collapsed ? 'w-[60px]' : 'w-[260px]'}">
  <!-- Top header bar -->
  <div class="flex items-center justify-between p-3 gap-2 border-b border-sidebar-border/40">
    {#if !collapsed}
      <!-- Collapse Icon on the LEFT -->
      <Button 
        variant="ghost" 
        size="icon"
        class="size-8 text-sidebar-foreground/60 hover:text-sidebar-foreground hover:bg-sidebar-accent rounded-lg shrink-0"
        onclick={() => collapsed = !collapsed}
        title="Collapse Sidebar"
      >
        <PanelLeftClose class="size-4" />
      </Button>

      <!-- Workspace Selector in middle -->
      <div class="flex-1 min-w-0">
        <Select.Root type="single" bind:value={activeWorkspaceId}>
          <Select.Trigger class="w-full h-8 text-xs font-semibold bg-sidebar text-sidebar-foreground border-sidebar-border truncate">
            {activeWorkspaceLabel}
          </Select.Trigger>
          <Select.Content class="bg-popover text-popover-foreground border-border">
            {#each workspaces as ws}
              <Select.Item value={ws.id} label={ws.name} class="text-xs font-medium cursor-pointer">
                {ws.name}
              </Select.Item>
            {/each}
          </Select.Content>
        </Select.Root>
      </div>

      <!-- Action button on the RIGHT -->
      {#if preferVoiceInput}
        <Button 
          variant="default" 
          size="icon"
          class="size-8 bg-primary text-primary-foreground hover:bg-primary/90 shadow font-bold rounded-lg shrink-0"
          onclick={() => handleNewSession(true)}
          title="New Voice Session"
        >
          <Mic class="size-4 stroke-[2.5]" />
        </Button>
      {:else}
        <Button 
          variant="ghost" 
          size="icon"
          class="size-8 text-sidebar-foreground/60 hover:text-sidebar-foreground hover:bg-sidebar-accent rounded-lg shrink-0"
          onclick={() => handleNewSession(false)}
          title="New Session"
        >
          <Plus class="size-4" />
        </Button>
      {/if}
    {:else}
      <!-- Collapsed header bar -->
      <div class="w-full flex items-center justify-between gap-1">
        <Button 
          variant="ghost" 
          size="icon"
          class="size-8 text-sidebar-foreground/60 hover:text-sidebar-foreground hover:bg-sidebar-accent rounded-lg shrink-0"
          onclick={() => collapsed = !collapsed}
          title="Expand Sidebar"
        >
          <PanelLeft class="size-4" />
        </Button>

        {#if preferVoiceInput}
          <Button 
            variant="default" 
            size="icon"
            class="size-8 bg-primary text-primary-foreground hover:bg-primary/90 shadow font-bold rounded-lg shrink-0"
            onclick={() => handleNewSession(true)}
            title="New Voice Session"
          >
            <Mic class="size-4 stroke-[2.5]" />
          </Button>
        {:else}
          <Button 
            variant="ghost" 
            size="icon"
            class="size-8 text-sidebar-foreground/60 hover:text-sidebar-foreground hover:bg-sidebar-accent rounded-lg shrink-0"
            onclick={() => handleNewSession(false)}
            title="New Session"
          >
            <Plus class="size-4" />
          </Button>
        {/if}
      </div>
    {/if}
  </div>

  <!-- Session List area -->
  <div class="flex-1 overflow-y-auto px-2 py-1">
    {#if collapsed}
      <div class="flex flex-col items-center gap-1">
        {#each store.sessions.filter(s => !s.archived) as session}
          <button
            onclick={() => {
              activeNav = 'sessions';
              store.selectSession(session.id);
            }}
            class="flex items-center justify-center size-8 rounded-lg transition-all relative {activeNav === 'sessions' && store.activeSessionId === session.id ? 'bg-sidebar-accent text-sidebar-accent-foreground' : 'hover:bg-sidebar-accent/50 text-sidebar-foreground/70'}"
            title={session.title}
          >
            <div class="relative">
              <MessageSquare class="size-3.5" />
              <span class="absolute -top-0.5 -right-0.5 size-1.5 rounded-full border border-sidebar {session.status === 'working' ? 'bg-blue-500' : session.status === 'waiting_for_input' ? 'bg-amber-400' : session.status === 'error' ? 'bg-destructive' : 'bg-emerald-500'}" title={session.status}></span>
            </div>
          </button>
        {/each}

        <!-- New Session item inside collapsed scrollable list -->
        <button
          onclick={() => handleNewSession(false)}
          class="flex items-center justify-center size-8 rounded-lg transition-all hover:bg-sidebar-accent/50 text-sidebar-foreground/70 hover:text-sidebar-foreground cursor-pointer"
          title="New Session"
        >
          <Plus class="size-3.5 text-sidebar-foreground/60" />
        </button>
      </div>
    {:else}
      <div class="flex flex-col gap-2">
        {#each groupedSessions as group}
          <div class="flex flex-col gap-0.5">
            <div class="text-[10px] font-semibold text-sidebar-foreground/50 uppercase tracking-wider px-2 py-0.5">
              {group.label}
            </div>
            <div class="flex flex-col gap-0.5">
              {#each group.items as session}
                <div
                  onclick={() => {
                    activeNav = 'sessions';
                    store.selectSession(session.id);
                  }}
                  onkeydown={(e) => {
                    if (e.key === 'Enter' || e.key === ' ') {
                      activeNav = 'sessions';
                      store.selectSession(session.id);
                    }
                  }}
                  role="button"
                  tabindex="0"
                  class="group flex items-center justify-between gap-2 px-2.5 py-1.5 rounded-lg text-xs text-left transition-all cursor-pointer {activeNav === 'sessions' && store.activeSessionId === session.id ? 'bg-sidebar-accent text-sidebar-accent-foreground font-medium' : 'hover:bg-sidebar-accent/50 text-sidebar-foreground/80'}"
                >
                  <div class="flex items-center gap-2.5 min-w-0 flex-1">
                    <div class="relative shrink-0">
                      <MessageSquare class="size-3.5 text-sidebar-foreground/60 group-hover:text-sidebar-foreground transition-colors" />
                      <span class="absolute -top-0.5 -right-0.5 size-1.5 rounded-full border border-sidebar {session.status === 'working' ? 'bg-blue-500' : session.status === 'waiting_for_input' ? 'bg-amber-400' : session.status === 'error' ? 'bg-destructive' : 'bg-emerald-500'}" title={session.status}></span>
                    </div>
                    <span class="truncate flex-1">{session.title}</span>
                  </div>

                  <!-- Archive button on hover -->
                  <button
                    onclick={(e) => {
                      e.stopPropagation();
                      store.archiveSession(session.id);
                    }}
                    class="opacity-0 group-hover:opacity-100 p-0.5 text-sidebar-foreground/50 hover:text-destructive hover:bg-sidebar-accent rounded transition-all shrink-0 cursor-pointer"
                    title="Archive Chat"
                  >
                    <Archive class="size-3" />
                  </button>
                </div>
              {/each}
            </div>
          </div>
        {/each}

        <!-- New Session item inside expanded scrollable list (no static background, compact spacing) -->
        <button
          onclick={() => handleNewSession(false)}
          class="flex items-center gap-2.5 px-2.5 py-1.5 rounded-lg text-xs text-left transition-all cursor-pointer hover:bg-sidebar-accent/50 text-sidebar-foreground/70 hover:text-sidebar-foreground font-medium"
          title="New Session"
        >
          <Plus class="size-3.5 text-sidebar-foreground/60 shrink-0" />
          <span class="truncate">New Session</span>
        </button>
      </div>
    {/if}
  </div>

  <!-- Bottom Navigation Row -->
  <div class="flex flex-col gap-0.5 p-2 border-t border-sidebar-border/40 bg-sidebar">
    <button
      onclick={() => activeNav = 'history'}
      class="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-xs transition-all {activeNav === 'history' ? 'bg-sidebar-accent text-sidebar-accent-foreground font-medium' : 'hover:bg-sidebar-accent/50 text-sidebar-foreground/70'}"
      title="History & Archives"
    >
      <History class="size-4 shrink-0" />
      {#if !collapsed}<span class="truncate text-left flex-1">History</span>{/if}
    </button>

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
