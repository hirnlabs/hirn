<script lang="ts">
  let theme = $state<'dark' | 'light'>('dark');
  let relayUrl = $state('https://agent.hirn-labs.com');
  let licenseKey = $state('');

  async function toggleTheme(newTheme: 'dark' | 'light') {
    theme = newTheme;
    if (typeof document !== 'undefined') {
      document.documentElement.setAttribute('data-theme', newTheme);
    }
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      await getCurrentWindow().setTheme(newTheme);
    } catch {
      // Running in standard web browser mode without Tauri runtime
    }
  }
</script>

<div class="subpage">
  <header class="subpage-header">
    <h2>Settings</h2>
    <p>Application preferences, theme appearance, and signaling relay configuration.</p>
  </header>

  <div class="settings-sections">
    <!-- Appearance Section -->
    <div class="setting-card">
      <div class="setting-info">
        <h3>Theme Appearance</h3>
        <p>Switch between dark and light themes (syncs native OS window title bar).</p>
      </div>
      <div class="theme-switcher">
        <button
          class="theme-btn {theme === 'dark' ? 'active' : ''}"
          onclick={() => toggleTheme('dark')}
        >
          Dark
        </button>
        <button
          class="theme-btn {theme === 'light' ? 'active' : ''}"
          onclick={() => toggleTheme('light')}
        >
          Light
        </button>
      </div>
    </div>

    <!-- Relay Endpoint Section -->
    <div class="setting-card">
      <div class="setting-info">
        <h3>Signaling Relay Endpoint</h3>
        <p>Configure the WebRTC signaling and encrypted store-and-forward relay URL (open-source and self-hostable).</p>
      </div>
      <div class="input-row">
        <input type="text" bind:value={relayUrl} class="text-input" placeholder="https://agent.hirn-labs.com" />
      </div>
    </div>

    <!-- Commercial License Section -->
    <div class="setting-card">
      <div class="setting-info">
        <h3>Commercial Licensing & Hirn Sync</h3>
        <p>Enter your Commercial License Key or Hirn Sync Subscription key.</p>
      </div>
      <div class="input-row">
        <input type="password" bind:value={licenseKey} class="text-input" placeholder="LICENSE-XXXX-XXXX-XXXX" />
        <button class="save-btn">Activate</button>
      </div>
    </div>
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
  .settings-sections { display: flex; flex-direction: column; gap: 16px; max-width: 680px; }
  .setting-card {
    background: var(--bg-card, #141417);
    border: 1px solid var(--border-color, #27272a);
    border-radius: 6px;
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .setting-info h3 { margin: 0 0 4px 0; font-size: 14px; font-weight: 600; }
  .setting-info p { margin: 0; font-size: 12px; color: var(--text-muted, #a1a1aa); }
  .theme-switcher { display: flex; gap: 8px; }
  .theme-btn {
    background: var(--bg-canvas, #18181b);
    border: 1px solid var(--border-color, #27272a);
    color: var(--text-muted, #a1a1aa);
    padding: 6px 16px;
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
  }
  .theme-btn.active {
    background: var(--border-color, #27272a);
    color: var(--text-main, #ffffff);
    font-weight: 600;
  }
  .input-row { display: flex; gap: 8px; }
  .text-input {
    flex: 1;
    background: var(--bg-canvas, #09090b);
    border: 1px solid var(--border-color, #27272a);
    color: var(--text-main, #f4f4f5);
    padding: 8px 12px;
    border-radius: 4px;
    font-size: 13px;
    font-family: ui-monospace, monospace;
    outline: none;
  }
  .save-btn {
    background: var(--bg-card, #27272a);
    border: 1px solid var(--border-color, #3f3f46);
    color: var(--text-main, #ffffff);
    padding: 0 16px;
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
  }
</style>
