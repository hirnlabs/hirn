<script lang="ts">
  import type { SessionStore } from '../stores/session.svelte';

  let { store }: { store: SessionStore } = $props();
</script>

<div class="subpage">
  <header class="subpage-header">
    <h2>ACP Agents</h2>
    <p>Manage connected Agent Client Protocol (ACP) executables and network endpoints.</p>
  </header>

  <div class="agent-list">
    {#each store.agents as agent}
      <div class="agent-card">
        <div class="agent-details">
          <div class="agent-name-row">
            <h3>{agent.name}</h3>
            {#if agent.isDefault}<span class="default-badge">Default</span>{/if}
          </div>
          <p class="desc">{agent.description}</p>
          <div class="meta-row">
            <span class="meta-item">Transport: <code>{agent.transportType}</code></span>
            <span class="meta-item">Target: <code>{agent.commandOrUrl}</code></span>
          </div>
        </div>
        <button class="test-btn">Test Connection</button>
      </div>
    {/each}
  </div>
</div>

<style>
  .subpage {
    flex: 1;
    height: 100vh;
    padding: 24px 32px;
    background: var(--bg-canvas, #09090b);
    color: var(--text-main, #e4e4e7);
    overflow-y: auto;
    box-sizing: border-box;
  }
  .subpage-header {
    border-bottom: 1px solid var(--border-color, #18181b);
    padding-bottom: 16px;
    margin-bottom: 24px;
  }
  .subpage-header h2 { margin: 0 0 4px 0; font-size: 18px; font-weight: 600; }
  .subpage-header p { margin: 0; font-size: 13px; color: var(--text-muted, #71717a); }
  .agent-list { display: flex; flex-direction: column; gap: 12px; }
  .agent-card {
    background: var(--bg-card, #141417);
    border: 1px solid var(--border-color, #27272a);
    border-radius: 6px;
    padding: 16px;
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .agent-name-row { display: flex; align-items: center; gap: 8px; }
  .agent-name-row h3 { margin: 0; font-size: 14px; font-weight: 600; }
  .default-badge {
    font-size: 10px;
    background: #27272a;
    color: #a1a1aa;
    padding: 2px 6px;
    border-radius: 4px;
    text-transform: uppercase;
  }
  .desc { font-size: 12px; color: var(--text-muted, #a1a1aa); margin: 6px 0 10px 0; }
  .meta-row { display: flex; gap: 16px; font-size: 11px; color: var(--text-muted, #71717a); }
  code { font-family: ui-monospace, monospace; color: var(--text-main, #e4e4e7); }
  .test-btn {
    background: var(--bg-card, #18181b);
    border: 1px solid var(--border-color, #27272a);
    color: var(--text-main, #e4e4e7);
    padding: 6px 12px;
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
  }
  .test-btn:hover { background: var(--border-color, #27272a); }
</style>
