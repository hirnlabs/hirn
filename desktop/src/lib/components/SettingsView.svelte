<script lang="ts">
  import { onMount } from 'svelte';
  import * as Card from '$lib/components/ui/card';
  import * as Field from '$lib/components/ui/field';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Separator } from '$lib/components/ui/separator';
  import * as ToggleGroup from '$lib/components/ui/toggle-group';

  let theme = $state<'dark' | 'light'>('light');
  let preferVoiceInput = $state(false);
  let relayUrl = $state('https://agent.hirn-labs.com');
  let licenseKey = $state('');

  onMount(() => {
    if (typeof document !== 'undefined') {
      const isDark = document.documentElement.classList.contains('dark') || 
                     document.documentElement.getAttribute('data-theme') === 'dark';
      theme = isDark ? 'dark' : 'light';
      preferVoiceInput = localStorage.getItem('hirn_prefer_voice_input') === 'true';
    }
  });

  async function toggleTheme(newTheme: 'dark' | 'light') {
    theme = newTheme;
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

  function toggleVoicePreference(val: boolean) {
    preferVoiceInput = val;
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('hirn_prefer_voice_input', String(val));
      window.dispatchEvent(new Event('storage'));
    }
  }
</script>

<div class="flex-1 h-screen overflow-y-auto bg-background text-foreground p-6 md:p-8 flex flex-col gap-6 select-none">
  <div>
    <h2 class="text-xl font-bold tracking-tight">Settings</h2>
    <p class="text-sm text-muted-foreground mt-1">Application preferences, theme appearance, and signaling relay configuration.</p>
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
