<script lang="ts">
  import { onMount } from 'svelte';
  import * as Card from '$lib/components/ui/card';
  import * as Field from '$lib/components/ui/field';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Separator } from '$lib/components/ui/separator';
  import * as ToggleGroup from '$lib/components/ui/toggle-group';
  import { CheckCircle2 } from '@lucide/svelte';
  import * as yaml from 'js-yaml';

  let theme = $state<'dark' | 'light'>('light');
  let preferVoiceInput = $state(false);
  let relayUrl = $state('https://agent.hirn-labs.com');
  let licenseKey = $state('');
  let saveNotification = $state<string | null>(null);

  async function loadSettingsYaml() {
    let yamlStr: string | null = null;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      yamlStr = await invoke<string>('load_named_config', { name: 'settings.yaml' });
    } catch {
      if (typeof localStorage !== 'undefined') {
        yamlStr = localStorage.getItem('hirn_settings_config');
      }
    }

    if (yamlStr) {
      try {
        const parsed: any = yaml.load(yamlStr);
        if (parsed && typeof parsed === 'object') {
          if (parsed.theme === 'dark' || parsed.theme === 'light') {
            theme = parsed.theme;
            applyTheme(parsed.theme);
          }
          if (typeof parsed.preferVoiceInput === 'boolean') preferVoiceInput = parsed.preferVoiceInput;
          if (typeof parsed.relayUrl === 'string') relayUrl = parsed.relayUrl;
          if (typeof parsed.licenseKey === 'string') licenseKey = parsed.licenseKey;
        }
      } catch (err) {
        console.warn('Failed to parse settings.yaml:', err);
      }
    } else {
      if (typeof document !== 'undefined') {
        const isDark = document.documentElement.classList.contains('dark') || 
                       document.documentElement.getAttribute('data-theme') === 'dark';
        theme = isDark ? 'dark' : 'light';
        preferVoiceInput = localStorage.getItem('hirn_prefer_voice_input') === 'true';
      }
    }
  }

  async function saveSettingsYaml() {
    const configObj = {
      theme,
      preferVoiceInput,
      relayUrl,
      licenseKey
    };

    const yamlStr = yaml.dump(configObj, { indent: 2 });
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('save_named_config', { name: 'settings.yaml', content: yamlStr });
      saveNotification = 'Saved ~/.hirn/config/settings.yaml';
      setTimeout(() => saveNotification = null, 2500);
    } catch {
      if (typeof localStorage !== 'undefined') {
        localStorage.setItem('hirn_settings_config', yamlStr);
        saveNotification = 'Saved settings (localStorage)';
        setTimeout(() => saveNotification = null, 2500);
      }
    }
  }

  onMount(() => {
    loadSettingsYaml();
  });

  async function applyTheme(newTheme: 'dark' | 'light') {
    if (typeof document !== 'undefined') {
      document.documentElement.setAttribute('data-theme', newTheme);
      document.documentElement.classList.toggle('dark', newTheme === 'dark');
    }
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      await getCurrentWindow().setTheme(newTheme);
    } catch {
      // Running in standard web browser mode without Tauri runtime
    }
  }

  async function toggleTheme(newTheme: 'dark' | 'light') {
    theme = newTheme;
    await applyTheme(newTheme);
    saveSettingsYaml();
  }

  function toggleVoicePreference(val: boolean) {
    preferVoiceInput = val;
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('hirn_prefer_voice_input', String(val));
      window.dispatchEvent(new Event('storage'));
    }
    saveSettingsYaml();
  }
</script>

<div class="flex-1 h-screen overflow-y-auto bg-background text-foreground p-6 md:p-8 flex flex-col gap-6 select-none">
  <div class="flex flex-col md:flex-row md:items-center justify-between gap-4">
    <div>
      <h2 class="text-xl font-bold tracking-tight">Settings</h2>
      <p class="text-sm text-muted-foreground mt-1">Application preferences, theme appearance, and signaling relay configuration.</p>
    </div>
    <div class="flex items-center gap-2 self-start md:self-auto">
      {#if saveNotification}
        <span class="text-xs font-mono text-emerald-400 bg-emerald-500/10 border border-emerald-500/30 px-3 py-1.5 rounded-lg animate-in fade-in duration-200">
          ✓ {saveNotification}
        </span>
      {/if}
      <Button onclick={saveSettingsYaml} variant="outline" size="sm" class="gap-1.5 cursor-pointer font-semibold shadow-xs">
        <CheckCircle2 class="size-4 text-emerald-500" />
        <span>Save Settings</span>
      </Button>
    </div>
  </div>

  <Separator />

  <Field.FieldGroup class="flex flex-col gap-6 max-w-2xl">
    <!-- Appearance Section -->
    <Card.Root>
      <Card.Header>
        <Card.Title class="text-base font-semibold">Theme Appearance</Card.Title>
        <Card.Description class="text-xs text-muted-foreground">Switch between dark and light themes (syncs native OS window title bar).</Card.Description>
      </Card.Header>
      <Card.Content>
        <ToggleGroup.Root
          type="single"
          value={theme}
          onValueChange={(val) => { if (val) toggleTheme(val as 'dark' | 'light'); }}
          class="justify-start"
        >
          <ToggleGroup.Item value="dark" class="px-4 text-xs font-medium">Dark</ToggleGroup.Item>
          <ToggleGroup.Item value="light" class="px-4 text-xs font-medium">Light</ToggleGroup.Item>
        </ToggleGroup.Root>
      </Card.Content>
    </Card.Root>

    <!-- Voice Input Preference Section -->
    <Card.Root>
      <Card.Header>
        <Card.Title class="text-base font-semibold">Default Input Mode</Card.Title>
        <Card.Description class="text-xs text-muted-foreground">Prefer voice input mode by default. Replaces the new chat button in the sidebar header with a bold voice action button with backdrop.</Card.Description>
      </Card.Header>
      <Card.Content>
        <ToggleGroup.Root
          type="single"
          value={preferVoiceInput ? 'voice' : 'text'}
          onValueChange={(val) => { if (val) toggleVoicePreference(val === 'voice'); }}
          class="justify-start"
        >
          <ToggleGroup.Item value="text" class="px-4 text-xs font-medium">Text Input</ToggleGroup.Item>
          <ToggleGroup.Item value="voice" class="px-4 text-xs font-medium">Voice Input</ToggleGroup.Item>
        </ToggleGroup.Root>
      </Card.Content>
    </Card.Root>

    <!-- Relay Endpoint Section -->
    <Card.Root>
      <Card.Header>
        <Card.Title class="text-base font-semibold">Signaling Relay Endpoint</Card.Title>
        <Card.Description class="text-xs text-muted-foreground">Configure the WebRTC signaling and encrypted store-and-forward relay URL (open-source and self-hostable).</Card.Description>
      </Card.Header>
      <Card.Content>
        <Field.Field>
          <Input type="text" bind:value={relayUrl} placeholder="https://agent.hirn-labs.com" class="font-mono text-xs flex-1" />
        </Field.Field>
      </Card.Content>
    </Card.Root>

    <!-- Commercial License Section -->
    <Card.Root>
      <Card.Header>
        <Card.Title class="text-base font-semibold">Commercial Licensing & Hirn Sync</Card.Title>
        <Card.Description class="text-xs text-muted-foreground">Enter your Commercial License Key or Hirn Sync Subscription key.</Card.Description>
      </Card.Header>
      <Card.Content>
        <Field.Field class="flex items-center gap-3">
          <Input type="password" bind:value={licenseKey} placeholder="LICENSE-XXXX-XXXX-XXXX" class="font-mono text-xs flex-1" />
          <Button size="sm">Activate</Button>
        </Field.Field>
      </Card.Content>
    </Card.Root>
  </Field.FieldGroup>
</div>
