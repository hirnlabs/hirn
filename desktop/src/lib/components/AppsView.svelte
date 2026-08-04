<script lang="ts">
  import * as Card from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import { Separator } from '$lib/components/ui/separator';

  let apps = $state([
    { id: 'file-browser', name: 'File Explorer', description: 'Browse and manage local files', author: 'Hirn Labs', status: 'installed' },
    { id: 'graph-view', name: 'Grafeo Graph', description: 'Interactive knowledge graph visualizer', author: 'Hirn Labs', status: 'installed' },
    { id: 'python-repl', name: 'Python Sandbox', description: 'Execute Python code in isolated container', author: 'Community', status: 'available' }
  ]);
</script>

<div class="flex-1 h-screen overflow-y-auto bg-background text-foreground p-6 md:p-8 flex flex-col gap-6 select-none">
  <div>
    <h2 class="text-xl font-bold tracking-tight">MCP Apps & Tools</h2>
    <p class="text-sm text-muted-foreground mt-1">Extensible tool UIs running inside sandboxed zero-port (<code class="font-mono text-xs">hirn://</code>) containers.</p>
  </div>

  <Separator />

  <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
    {#each apps as app}
      <Card.Root class="flex flex-col justify-between">
        <Card.Header class="pb-3">
          <div class="flex items-center justify-between">
            <Card.Title class="text-base font-semibold">{app.name}</Card.Title>
            <Badge variant={app.status === 'installed' ? 'default' : 'outline'} class="text-[10px] uppercase font-mono tracking-wider">
              {app.status}
            </Badge>
          </div>
          <span class="text-xs text-muted-foreground font-mono">by {app.author}</span>
        </Card.Header>
        <Card.Content>
          <Card.Description class="text-xs text-muted-foreground">{app.description}</Card.Description>
        </Card.Content>
      </Card.Root>
    {/each}
  </div>
</div>
