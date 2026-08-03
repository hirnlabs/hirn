<script lang="ts">
  let models = $state([
    { name: 'llama3.2:3b-instruct-q4_K_M', size: '2.0 GB', backend: 'Ollama / Local', status: 'loaded' },
    { name: 'qwen2.5-coder:7b-instruct', size: '4.7 GB', backend: 'Server / RPC Sharded', status: 'available' },
    { name: 'whisper-small.en', size: '480 MB', backend: 'Transcribe Local', status: 'loaded' }
  ]);
</script>

<div class="subpage">
  <header class="subpage-header">
    <h2>Local Models & Inference Router</h2>
    <p>Status of local GGUF models, VRAM memory pools, and Router model dispatch.</p>
  </header>

  <div class="model-list">
    {#each models as model}
      <div class="model-card">
        <div class="model-info">
          <h3>{model.name}</h3>
          <div class="meta-row">
            <span>Backend: <code>{model.backend}</code></span>
            <span>Memory: <code>{model.size}</code></span>
          </div>
        </div>
        <span class="status-badge {model.status}">{model.status}</span>
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
  .model-list { display: flex; flex-direction: column; gap: 12px; }
  .model-card {
    background: var(--bg-card, #141417);
    border: 1px solid var(--border-color, #27272a);
    border-radius: 6px;
    padding: 16px;
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .model-info h3 { margin: 0 0 6px 0; font-size: 14px; font-weight: 600; font-family: ui-monospace, monospace; }
  .meta-row { display: flex; gap: 16px; font-size: 12px; color: var(--text-muted, #71717a); }
  code { font-family: ui-monospace, monospace; color: var(--text-main, #e4e4e7); }
  .status-badge {
    font-size: 10px;
    text-transform: uppercase;
    padding: 2px 6px;
    border-radius: 4px;
    border: 1px solid var(--border-color, #27272a);
    color: var(--text-muted, #a1a1aa);
  }
  .status-badge.loaded { color: #10b981; border-color: #10b98130; }
</style>
