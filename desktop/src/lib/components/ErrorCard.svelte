<script lang="ts">
  import type { ChatMessage } from '../types/acp';
  import * as Alert from '$lib/components/ui/alert';
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import { OctagonAlert, Terminal, Cpu, Network, ShieldAlert, ChevronDown, Copy, Check } from '@lucide/svelte';

  let { message }: { message: ChatMessage } = $props();

  let details = $derived(message.errorDetails);
  let isExpanded = $state(false);
  let isCopied = $state(false);

  function getBadgeConfig(source?: string) {
    switch (source) {
      case 'stdio':
        return { label: 'STDIO Stderr', icon: Terminal, variant: 'outline' as const };
      case 'jsonrpc':
        return { label: 'ACP JSON-RPC', icon: Cpu, variant: 'destructive' as const };
      case 'ipc':
        return { label: 'IPC Transport', icon: Network, variant: 'secondary' as const };
      case 'connection':
        return { label: 'Agent Connection', icon: ShieldAlert, variant: 'outline' as const };
      default:
        return { label: 'Agent Error', icon: OctagonAlert, variant: 'destructive' as const };
    }
  }

  let badge = $derived(getBadgeConfig(details?.source));

  function copyRawPayload() {
    if (!details?.rawPayload) return;
    navigator.clipboard.writeText(details.rawPayload);
    isCopied = true;
    setTimeout(() => { isCopied = false; }, 2000);
  }
</script>

<Alert.Root variant="destructive" class="my-2 border-destructive/30 bg-destructive/5 shadow-none rounded-xl p-3.5">
  <OctagonAlert class="size-4 text-destructive" />
  
  <div class="flex flex-col gap-1.5 w-full">
    <!-- Header row -->
    <div class="flex items-center justify-between gap-2">
      <div class="flex items-center gap-2 flex-wrap">
        <Alert.Title class="text-xs font-semibold tracking-tight text-foreground">System / Agent Error</Alert.Title>
        {#if badge}
          <Badge variant={badge.variant} class="text-[9px] font-mono py-0 px-1.5 gap-1 h-4">
            <badge.icon data-icon="inline-start" />
            <span>{badge.label}</span>
          </Badge>
        {/if}
      </div>
      <span class="text-[10px] font-mono text-muted-foreground">{new Date(message.timestamp).toLocaleTimeString()}</span>
    </div>

    <!-- Error message text -->
    <Alert.Description class="text-xs font-mono font-medium leading-relaxed text-foreground/90">
      {#if details?.code !== undefined}
        <span class="text-destructive font-bold font-mono mr-1">[Code {details.code}]</span>
      {/if}
      {message.content}
    </Alert.Description>

    <!-- Clean single-line metadata summary (no grid cards or heavy borders) -->
    {#if details?.command || details?.method || details?.pid}
      <div class="flex items-center gap-2 text-[10px] font-mono text-muted-foreground pt-0.5 flex-wrap">
        {#if details.command}
          <span class="truncate max-w-[240px]" title={details.command}>{details.command}</span>
        {/if}
        {#if details.command && details.method}
          <span>•</span>
        {/if}
        {#if details.method}
          <span class="text-foreground/80 font-semibold">{details.method}</span>
        {/if}
        {#if (details.command || details.method) && details.pid}
          <span>•</span>
        {/if}
        {#if details.pid}
          <span>PID {details.pid}</span>
        {/if}
      </div>
    {/if}

    <!-- Minimal raw payload toggle -->
    {#if details?.rawPayload}
      <div class="pt-1.5 flex flex-col gap-1.5">
        <Button
          variant="ghost"
          size="sm"
          onclick={() => isExpanded = !isExpanded}
          class="h-6 px-1 justify-start text-[10px] text-muted-foreground hover:text-foreground font-mono w-auto self-start gap-1 cursor-pointer"
        >
          <Terminal data-icon="inline-start" class="size-3" />
          <span>{isExpanded ? 'Hide Payload' : 'Show Payload'}</span>
          <ChevronDown class="size-3 transition-transform {isExpanded ? 'rotate-180' : ''}" />
        </Button>

        {#if isExpanded}
          <div class="relative">
            <pre class="p-2.5 bg-muted/60 text-foreground rounded-lg text-[10px] font-mono overflow-x-auto max-h-40 leading-relaxed select-text">{details.rawPayload}</pre>
            <Button
              variant="ghost"
              size="icon"
              onclick={copyRawPayload}
              class="absolute top-1.5 right-1.5 size-6 hover:bg-background"
              title="Copy payload"
            >
              {#if isCopied}
                <Check class="size-3 text-primary" />
              {:else}
                <Copy class="size-3 text-muted-foreground" />
              {/if}
            </Button>
          </div>
        {/if}
      </div>
    {/if}
  </div>
</Alert.Root>
