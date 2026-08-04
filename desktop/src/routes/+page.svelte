<script lang="ts">
  import { SessionStore } from '../lib/stores/session.svelte';
  import * as Sidebar from '$lib/components/ui/sidebar';
  import AppSidebar from '../lib/components/Sidebar.svelte';
  import ChatCanvas from '../lib/components/ChatCanvas.svelte';
  import AppsView from '../lib/components/AppsView.svelte';
  import AgentsView from '../lib/components/AgentsView.svelte';
  import ModelsView from '../lib/components/ModelsView.svelte';
  import SettingsView from '../lib/components/SettingsView.svelte';
  import HistoryView from '../lib/components/HistoryView.svelte';

  const store = new SessionStore();
  let activeNav = $state<'sessions' | 'apps' | 'agents' | 'models' | 'settings' | 'history'>('sessions');
</script>

<div class="flex w-screen h-screen overflow-hidden bg-background text-foreground">
  <Sidebar.Provider>
    <AppSidebar {store} bind:activeNav />

    <Sidebar.Inset class="flex-1 h-screen overflow-hidden flex flex-col">
      {#if activeNav === 'sessions'}
        <ChatCanvas {store} />
      {:else if activeNav === 'apps'}
        <AppsView />
      {:else if activeNav === 'agents'}
        <AgentsView {store} />
      {:else if activeNav === 'models'}
        <ModelsView />
      {:else if activeNav === 'history'}
        <HistoryView {store} bind:activeNav />
      {:else if activeNav === 'settings'}
        <SettingsView />
      {/if}
    </Sidebar.Inset>
  </Sidebar.Provider>
</div>
