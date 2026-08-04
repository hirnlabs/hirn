<script lang="ts">
  import type { SessionStore } from '../stores/session.svelte';
  import * as Card from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import { Separator } from '$lib/components/ui/separator';

  let { store }: { store: SessionStore } = $props();
</script>

<div class="flex-1 h-screen overflow-y-auto bg-background text-foreground p-6 md:p-8 flex flex-col gap-6 select-none">
  <div>
    <h2 class="text-xl font-bold tracking-tight">ACP Agents</h2>
    <p class="text-sm text-muted-foreground mt-1">Manage connected Agent Client Protocol (ACP) executables and network endpoints.</p>
  </div>

  <Separator />

  <div class="flex flex-col gap-4">
    {#each store.agents as agent}
      <Card.Root>
        <Card.Header class="flex flex-row items-center justify-between gap-2 pb-2">
          <div class="flex items-center gap-2">
            <Card.Title class="text-base font-semibold">{agent.name}</Card.Title>
            {#if agent.isDefault}
              <Badge variant="secondary" class="text-[10px] uppercase font-mono tracking-wider">Default</Badge>
            {/if}
          </div>
          <Button variant="outline" size="sm">Test Connection</Button>
        </Card.Header>
        <Card.Content class="pt-0">
          <Card.Description class="text-xs text-muted-foreground mb-3">{agent.description}</Card.Description>
          <div class="flex flex-wrap gap-4 text-xs text-muted-foreground">
            <span>Transport: <code class="font-mono text-foreground font-medium">{agent.transportType}</code></span>
            <span>Target: <code class="font-mono text-foreground font-medium">{agent.commandOrUrl}</code></span>
          </div>
        </Card.Content>
      </Card.Root>
    {/each}
  </div>
</div>
