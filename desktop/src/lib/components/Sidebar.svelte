<script lang="ts">
  import { onMount } from 'svelte';
  import type { SessionStore } from '../stores/session.svelte';
  import * as Sidebar from '$lib/components/ui/sidebar';
  import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
  import * as Dialog from '$lib/components/ui/dialog';
  import * as Field from '$lib/components/ui/field';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { 
    MessageSquare, 
    Plus, 
    Mic, 
    Settings, 
    Cpu, 
    Bot, 
    Boxes, 
    Archive, 
    History, 
    Folder, 
    ChevronsUpDown, 
    Check 
  } from '@lucide/svelte';

  let { store, activeNav = $bindable('sessions') }: {
    store: SessionStore;
    activeNav: 'sessions' | 'apps' | 'agents' | 'models' | 'settings' | 'history';
  } = $props();

  const sidebar = Sidebar.useSidebar();

  let preferVoiceInput = $state(false);

  let workspaces = $state([
    { id: 'ws-1', name: 'hirnlabs/hirn', path: '.hirn/workspaces/hirnlabs-hirn' },
    { id: 'ws-2', name: 'Personal Projects', path: '.hirn/workspaces/personal-projects' },
    { id: 'ws-3', name: 'Research Lab', path: '.hirn/workspaces/research-lab' }
  ]);
  let activeWorkspaceId = $state('ws-1');

  let isCreateWorkspaceDialogOpen = $state(false);
  let newWsName = $state('');
  let newWsFolder = $state('');
  let userCustomizedFolder = $state(false);

  function handleNameInput(e: Event) {
    const name = (e.target as HTMLInputElement).value;
    newWsName = name;
    if (!userCustomizedFolder) {
      const slug = name.toLowerCase().trim().replace(/[^a-z0-9_-]/g, '-').replace(/-+/g, '-');
      newWsFolder = slug ? `.hirn/workspaces/${slug}` : '';
    }
  }

  function handleCreateWorkspace() {
    if (!newWsName.trim()) return;
    const slug = newWsName.toLowerCase().trim().replace(/[^a-z0-9_-]/g, '-').replace(/-+/g, '-');
    const path = newWsFolder.trim() || `.hirn/workspaces/${slug || 'workspace'}`;
    const newWs = {
      id: `ws-${Date.now()}`,
      name: newWsName.trim(),
      path
    };
    workspaces.push(newWs);
    activeWorkspaceId = newWs.id;
    newWsName = '';
    newWsFolder = '';
    userCustomizedFolder = false;
    isCreateWorkspaceDialogOpen = false;
  }

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
    const unarchived = store.sessions.filter(s => !s.archived && !s.isDraft);

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

<Sidebar.Root collapsible="icon" class="select-none">
  <!-- Header with SidebarTrigger, Workspace Dropdown, and Action Button in single title bar row -->
  <Sidebar.Header class="h-12 p-2 flex-row items-center gap-1.5 border-b border-sidebar-border/40 shrink-0">
    <Sidebar.Trigger class="size-8 text-sidebar-foreground/60 hover:text-sidebar-foreground hover:bg-sidebar-accent rounded-lg shrink-0" />

    {#if sidebar.state === 'expanded'}
      <!-- Workspace Selector Dropdown in Middle -->
      <div class="flex-1 min-w-0">
        <DropdownMenu.Root>
          <DropdownMenu.Trigger class="w-full h-8 text-xs font-normal bg-sidebar hover:bg-sidebar-accent/60 text-sidebar-foreground border border-sidebar-border/80 rounded-lg px-2 flex items-center justify-between gap-1 transition-colors outline-none cursor-pointer">
            <div class="flex items-center gap-1.5 min-w-0 flex-1">
              <Folder class="size-3.5 text-sidebar-foreground/60 shrink-0" />
              <span class="truncate font-medium text-left">{activeWorkspaceLabel}</span>
            </div>
            <ChevronsUpDown class="size-3 text-sidebar-foreground/50 shrink-0" />
          </DropdownMenu.Trigger>

          <DropdownMenu.Content align="center" class="w-56 bg-popover text-popover-foreground border-border shadow-xl rounded-xl p-1 z-50">
            <DropdownMenu.Group>
              <DropdownMenu.GroupHeading class="text-[10px] font-semibold text-muted-foreground uppercase tracking-wider px-2 py-1 font-mono">
                Workspaces
              </DropdownMenu.GroupHeading>
              {#each workspaces as ws}
                <DropdownMenu.Item 
                  onclick={() => activeWorkspaceId = ws.id}
                  class="flex items-center justify-between px-2 py-1.5 text-xs rounded-lg cursor-pointer hover:bg-accent hover:text-accent-foreground transition-colors {activeWorkspaceId === ws.id ? 'bg-accent/60 font-medium' : ''}"
                >
                  <div class="flex items-center gap-2 min-w-0 flex-1">
                    <Folder class="size-3.5 text-muted-foreground shrink-0" />
                    <span class="truncate font-normal">{ws.name}</span>
                  </div>
                  {#if activeWorkspaceId === ws.id}
                    <Check class="size-3.5 text-primary shrink-0 ml-1" />
                  {/if}
                </DropdownMenu.Item>
              {/each}
            </DropdownMenu.Group>

            <DropdownMenu.Separator class="my-1 bg-border/60" />

            <DropdownMenu.Item 
              onclick={() => isCreateWorkspaceDialogOpen = true}
              class="flex items-center gap-2 px-2 py-1.5 text-xs font-medium text-primary hover:bg-primary/10 rounded-lg cursor-pointer transition-colors"
            >
              <Plus class="size-3.5 shrink-0" />
              <span>Create New Workspace...</span>
            </DropdownMenu.Item>
          </DropdownMenu.Content>
        </DropdownMenu.Root>
      </div>

      <!-- Action Button on Right -->
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
    {/if}
  </Sidebar.Header>

  <!-- Sidebar Content with Session Groups -->
  <Sidebar.Content>
    {#each groupedSessions as group}
      <Sidebar.Group>
        <Sidebar.GroupLabel class="text-[10px] font-medium text-sidebar-foreground/50 uppercase tracking-wider font-mono">
          {group.label}
        </Sidebar.GroupLabel>
        <Sidebar.GroupContent>
          <Sidebar.Menu>
            {#each group.items as session}
              <Sidebar.MenuItem>
                <Sidebar.MenuButton
                  isActive={activeNav === 'sessions' && store.activeSessionId === session.id}
                  onclick={() => {
                    activeNav = 'sessions';
                    store.selectSession(session.id);
                  }}
                  tooltipContent={session.title}
                  class="group/item flex items-center justify-between text-xs font-medium"
                >
                  <div class="flex items-center gap-2.5 min-w-0 flex-1">
                    <div class="relative shrink-0">
                      <MessageSquare class="size-4 text-sidebar-foreground/70 group-hover/item:text-sidebar-foreground transition-colors" />
                      <span class="absolute -top-0.5 -right-0.5 size-1.5 rounded-full border border-sidebar {session.status === 'working' ? 'bg-primary' : session.status === 'waiting_for_input' ? 'bg-chart-2' : session.status === 'error' ? 'bg-destructive' : 'bg-primary/70'}" title={session.status}></span>
                    </div>
                    <span class="truncate text-xs font-medium">{session.title}</span>
                  </div>

                  {#if sidebar.state === 'expanded'}
                    <Sidebar.MenuAction
                      onclick={(e) => {
                        e.stopPropagation();
                        store.archiveSession(session.id);
                      }}
                      title="Archive Chat"
                      class="opacity-0 group-hover/item:opacity-100 hover:text-destructive transition-opacity"
                    >
                      <Archive class="size-3.5" />
                    </Sidebar.MenuAction>
                  {/if}
                </Sidebar.MenuButton>
              </Sidebar.MenuItem>
            {/each}
          </Sidebar.Menu>
        </Sidebar.GroupContent>
      </Sidebar.Group>
    {/each}

    <!-- New Session Action -->
    <Sidebar.Group>
      <Sidebar.Menu>
        <Sidebar.MenuItem>
          <Sidebar.MenuButton
            onclick={() => handleNewSession(false)}
            tooltipContent="New Session"
            class="text-xs font-medium"
          >
            <Plus class="size-4 text-sidebar-foreground/70 shrink-0" />
            <span class="text-xs font-medium">New Session</span>
          </Sidebar.MenuButton>
        </Sidebar.MenuItem>
      </Sidebar.Menu>
    </Sidebar.Group>
  </Sidebar.Content>

  <!-- Sidebar Footer with Bottom Navigation -->
  <Sidebar.Footer class="border-t border-sidebar-border/40 p-2">
    <Sidebar.Menu>
      <Sidebar.MenuItem>
        <Sidebar.MenuButton
          isActive={activeNav === 'history'}
          onclick={() => activeNav = 'history'}
          tooltipContent="History & Archives"
          class="text-xs font-medium"
        >
          <History class="size-4 shrink-0" />
          <span class="text-xs font-medium">History</span>
        </Sidebar.MenuButton>
      </Sidebar.MenuItem>

      <Sidebar.MenuItem>
        <Sidebar.MenuButton
          isActive={activeNav === 'apps'}
          onclick={() => activeNav = 'apps'}
          tooltipContent="Apps, Tools & Skills"
          class="text-xs font-medium"
        >
          <Boxes class="size-4 shrink-0" />
          <span class="text-xs font-medium">Apps, Tools & Skills</span>
        </Sidebar.MenuButton>
      </Sidebar.MenuItem>

      <Sidebar.MenuItem>
        <Sidebar.MenuButton
          isActive={activeNav === 'agents'}
          onclick={() => activeNav = 'agents'}
          tooltipContent="Agents & Providers"
          class="text-xs font-medium"
        >
          <Bot class="size-4 shrink-0" />
          <span class="text-xs font-medium">Agents & Providers</span>
        </Sidebar.MenuButton>
      </Sidebar.MenuItem>

      <Sidebar.MenuItem>
        <Sidebar.MenuButton
          isActive={activeNav === 'models'}
          onclick={() => activeNav = 'models'}
          tooltipContent="Model Configuration"
          class="text-xs font-medium"
        >
          <Cpu class="size-4 shrink-0" />
          <span class="text-xs font-medium">Model Configuration</span>
        </Sidebar.MenuButton>
      </Sidebar.MenuItem>

      <Sidebar.MenuItem>
        <Sidebar.MenuButton
          isActive={activeNav === 'settings'}
          onclick={() => activeNav = 'settings'}
          tooltipContent="Settings"
          class="text-xs font-medium"
        >
          <Settings class="size-4 shrink-0" />
          <span class="text-xs font-medium">Settings</span>
        </Sidebar.MenuButton>
      </Sidebar.MenuItem>
    </Sidebar.Menu>
  </Sidebar.Footer>

  <Sidebar.Rail />
</Sidebar.Root>

<!-- Create Workspace Dialog Modal -->
<Dialog.Root bind:open={isCreateWorkspaceDialogOpen}>
  <Dialog.Content class="sm:max-w-[425px]">
    <Dialog.Header>
      <Dialog.Title class="text-base font-bold">Create Workspace</Dialog.Title>
      <Dialog.Description class="text-xs text-muted-foreground">
        Set up a new workspace context for your agent sessions, local search index, and document sync.
      </Dialog.Description>
    </Dialog.Header>

    <Field.FieldGroup class="flex flex-col gap-4 py-3">
      <Field.Field>
        <Field.FieldLabel for="ws-name" class="text-xs font-medium">Workspace Name</Field.FieldLabel>
        <Input 
          id="ws-name" 
          type="text"
          value={newWsName}
          oninput={handleNameInput}
          placeholder="e.g. My Project" 
          class="text-xs"
        />
      </Field.Field>

      <Field.Field>
        <Field.FieldLabel for="ws-folder" class="text-xs font-medium">Folder Location</Field.FieldLabel>
        <Input 
          id="ws-folder" 
          type="text"
          bind:value={newWsFolder}
          oninput={() => userCustomizedFolder = true}
          placeholder=".hirn/workspaces/my-project" 
          class="font-mono text-xs"
        />
        <Field.FieldDescription class="text-[11px] text-muted-foreground">
          Default location: <code class="font-mono text-foreground font-medium">.hirn/workspaces/{newWsName ? newWsName.toLowerCase().trim().replace(/[^a-z0-9_-]/g, '-').replace(/-+/g, '-') : '<name>'}</code>
        </Field.FieldDescription>
      </Field.Field>
    </Field.FieldGroup>

    <Dialog.Footer class="gap-2 pt-2">
      <Button variant="outline" size="sm" onclick={() => isCreateWorkspaceDialogOpen = false}>Cancel</Button>
      <Button size="sm" disabled={!newWsName.trim()} onclick={handleCreateWorkspace}>Create Workspace</Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
