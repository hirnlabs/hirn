<script lang="ts">
  import { SessionStore } from '../lib/stores/session.svelte';
  import Sidebar from '../lib/components/Sidebar.svelte';
  import ChatCanvas from '../lib/components/ChatCanvas.svelte';
  import AppsView from '../lib/components/AppsView.svelte';
  import AgentsView from '../lib/components/AgentsView.svelte';
  import ModelsView from '../lib/components/ModelsView.svelte';
  import SettingsView from '../lib/components/SettingsView.svelte';

  const store = new SessionStore();
  let activeNav = $state<'sessions' | 'apps' | 'agents' | 'models' | 'settings'>('sessions');
</script>

<div class="app-layout">
  <Sidebar {store} bind:activeNav />

  {#if activeNav === 'sessions'}
    <ChatCanvas {store} />
  {:else if activeNav === 'apps'}
    <AppsView />
  {:else if activeNav === 'agents'}
    <AgentsView {store} />
  {:else if activeNav === 'models'}
    <ModelsView />
  {:else if activeNav === 'settings'}
    <SettingsView />
  {/if}
</div>

<style>
  :global(:root) {
    --bg-canvas: #09090b;
    --bg-sidebar: #09090b;
    --bg-card: #141417;
    --border-color: #18181b;
    --text-main: #f4f4f5;
    --text-muted: #71717a;
    color-scheme: dark;
  }

  :global(html[data-theme="light"]) {
    --bg-canvas: #ffffff;
    --bg-sidebar: #f4f4f5;
    --bg-card: #f4f4f5;
    --border-color: #e4e4e7;
    --text-main: #09090b;
    --text-muted: #71717a;
    color-scheme: light;
  }

  :global(body) {
    margin: 0;
    padding: 0;
    font-family: Inter, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    background-color: var(--bg-canvas);
    color: var(--text-main);
    overflow: hidden;
  }

  .app-layout {
    display: flex;
    width: 100vw;
    height: 100vh;
  }
</style>
