<script lang="ts">
  import type { ToolCall } from '../types/acp';

  let { toolCall }: { toolCall: ToolCall } = $props();
  let expanded = $state(false);
</script>

<div class="tool-card {toolCall.status}">
  <button class="tool-header" onclick={() => expanded = !expanded}>
    <span class="status-indicator">
      {#if toolCall.status === 'running'}
        <span class="pulse">●</span>
      {:else if toolCall.status === 'completed'}
        <span class="done">✓</span>
      {:else if toolCall.status === 'failed'}
        <span class="fail">✕</span>
      {:else}
        <span class="idle">○</span>
      {/if}
    </span>
    <span class="tool-name">{toolCall.name}</span>
    <span class="chevron">{expanded ? '▲' : '▼'}</span>
  </button>

  {#if expanded}
    <div class="tool-body">
      <div class="args-section">
        <span class="section-label">Input Parameters</span>
        <pre><code>{JSON.stringify(toolCall.arguments, null, 2)}</code></pre>
      </div>
      {#if toolCall.result}
        <div class="result-section">
          <span class="section-label">Execution Result</span>
          <pre><code>{toolCall.result}</code></pre>
        </div>
      {/if}
      {#if toolCall.error}
        <div class="error-section">
          <span class="section-label">Error Output</span>
          <pre><code>{toolCall.error}</code></pre>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .tool-card {
    background: #141417;
    border: 1px solid #27272a;
    border-radius: 4px;
    margin: 8px 0;
    overflow: hidden;
    font-size: 12px;
  }
  .tool-header {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    background: transparent;
    border: none;
    color: #a1a1aa;
    cursor: pointer;
    text-align: left;
    font-family: inherit;
  }
  .tool-header:hover {
    background: #1f1f23;
    color: #f4f4f5;
  }
  .tool-name {
    font-weight: 500;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    flex: 1;
  }
  .chevron {
    font-size: 9px;
    color: #52525b;
  }
  .tool-body {
    padding: 8px 10px;
    background: #09090b;
    border-top: 1px solid #27272a;
  }
  .section-label {
    display: block;
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: #52525b;
    margin-bottom: 4px;
  }
  pre {
    margin: 0 0 6px 0;
    padding: 6px 8px;
    background: #121215;
    border-radius: 4px;
    overflow-x: auto;
    font-size: 11px;
    color: #e4e4e7;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  }
  .pulse { color: #ffffff; display: inline-block; animation: pulse 1s infinite; }
  .done { color: #a1a1aa; }
  .fail { color: #f87171; }
  .idle { color: #52525b; }
  @keyframes pulse { 0% { opacity: 0.3; } 50% { opacity: 1; } 100% { opacity: 0.3; } }
</style>
