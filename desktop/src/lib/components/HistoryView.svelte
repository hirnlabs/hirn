<script lang="ts">
  import type { SessionStore } from '../stores/session.svelte';
  import * as Card from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import { Separator } from '$lib/components/ui/separator';
  import { MessageSquare, Archive, RefreshCw, Clock } from '@lucide/svelte';

  let { store, activeNav = $bindable() }: { store: SessionStore; activeNav?: string } = $props();

  let activeSessions = $derived(store.sessions.filter(s => !s.archived));
  let archivedSessions = $derived(store.sessions.filter(s => s.archived));

  function openSession(id: string) {
    store.selectSession(id);
    if (activeNav) activeNav = 'sessions';
  }
</script>

<div class="flex-1 h-screen overflow-y-auto bg-background text-foreground p-6 md:p-8 flex flex-col gap-6 select-none">
  <div>
    <h2 class="text-xl font-bold tracking-tight">Chat History & Archives</h2>
    <p class="text-sm text-muted-foreground mt-1">Review past conversations, timestamps, and restore archived chat sessions.</p>
  </div>

  <Separator />

  <!-- Active Sessions Section -->
  <div class="flex flex-col gap-3">
    <h3 class="text-sm font-semibold text-foreground flex items-center gap-2">
      <Clock class="size-4 text-muted-foreground" />
      <span>Active Sessions ({activeSessions.length})</span>
    </h3>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
      {#each activeSessions as session (session.id)}
        <Card.Root class="hover:border-border transition-all">
          <Card.Header class="pb-2">
            <div class="flex items-center justify-between gap-2">
              <Card.Title class="text-sm font-semibold truncate flex-1">{session.title}</Card.Title>
              <Badge variant="outline" class="text-[10px] font-mono shrink-0">{session.agentName}</Badge>
            </div>
            <Card.Description class="text-xs text-muted-foreground">
              {new Date(session.updatedAt || session.createdAt).toLocaleString()} · {session.messages.length} messages
            </Card.Description>
          </Card.Header>
          <Card.Content class="flex items-center justify-end gap-2 pt-2">
            <Button variant="ghost" size="sm" class="text-xs text-muted-foreground hover:text-destructive" onclick={() => store.archiveSession(session.id)}>
              <Archive class="size-3.5 mr-1" />
              Archive
            </Button>
            <Button variant="secondary" size="sm" class="text-xs" onclick={() => openSession(session.id)}>
              <MessageSquare class="size-3.5 mr-1" />
              Open
            </Button>
          </Card.Content>
        </Card.Root>
      {:else}
        <p class="text-xs text-muted-foreground col-span-2 py-4">No active sessions found.</p>
      {/each}
    </div>
  </div>

  <!-- Archived Sessions Section -->
  {#if archivedSessions.length > 0}
    <div class="flex flex-col gap-3 mt-4">
      <h3 class="text-sm font-semibold text-foreground flex items-center gap-2">
        <Archive class="size-4 text-muted-foreground" />
        <span>Archived Sessions ({archivedSessions.length})</span>
      </h3>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
        {#each archivedSessions as session (session.id)}
          <Card.Root class="opacity-80 hover:opacity-100 transition-all">
            <Card.Header class="pb-2">
              <div class="flex items-center justify-between gap-2">
                <Card.Title class="text-sm font-semibold truncate flex-1">{session.title}</Card.Title>
                <Badge variant="secondary" class="text-[10px] font-mono shrink-0">Archived</Badge>
              </div>
              <Card.Description class="text-xs text-muted-foreground">
                {new Date(session.updatedAt || session.createdAt).toLocaleString()} · {session.messages.length} messages
              </Card.Description>
            </Card.Header>
            <Card.Content class="flex items-center justify-end gap-2 pt-2">
              <Button variant="outline" size="sm" class="text-xs" onclick={() => store.unarchiveSession(session.id)}>
                <RefreshCw class="size-3.5 mr-1" />
                Unarchive
              </Button>
            </Card.Content>
          </Card.Root>
        {/each}
      </div>
    </div>
  {/if}
</div>
