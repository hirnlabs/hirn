<script lang="ts">
  let apps = $state([
    { id: 'file-browser', name: 'File Explorer', description: 'Browse and manage local files', author: 'Hirn Labs', status: 'installed' },
    { id: 'graph-view', name: 'Grafeo Graph', description: 'Interactive knowledge graph visualizer', author: 'Hirn Labs', status: 'installed' },
    { id: 'python-repl', name: 'Python Sandbox', description: 'Execute Python code in isolated container', author: 'Community', status: 'available' }
  ]);
</script>

<div class="subpage">
  <header class="subpage-header">
    <h2>MCP Apps & Tools</h2>
    <p>Extensible tool UIs running inside sandboxed zero-port (`hirn://`) containers.</p>
  </header>

  <div class="app-grid">
    {#each apps as app}
      <div class="app-card">
        <div class="app-info">
          <h3>{app.name}</h3>
          <span class="author">by {app.author}</span>
          <p>{app.description}</p>
        </div>
        <div class="app-action">
          <span class="badge {app.status}">{app.status}</span>
        </div>
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
  .app-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
    gap: 16px;
  }
  .app-card {
    background: var(--bg-card, #141417);
    border: 1px solid var(--border-color, #27272a);
    border-radius: 6px;
    padding: 16px;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
  }
  .app-info h3 { margin: 0 0 2px 0; font-size: 14px; font-weight: 600; }
  .author { font-size: 11px; color: var(--text-muted, #52525b); }
  .app-info p { font-size: 12px; color: var(--text-muted, #a1a1aa); margin: 10px 0 16px 0; }
  .badge {
    font-size: 10px;
    text-transform: uppercase;
    padding: 2px 6px;
    border-radius: 4px;
    border: 1px solid var(--border-color, #27272a);
    color: var(--text-muted, #a1a1aa);
  }
  .badge.installed { color: #10b981; border-color: #10b98130; }
</style>
