<script lang="ts">
  import * as Card from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import { Separator } from '$lib/components/ui/separator';

  let models = $state([
    { name: 'llama3.2:3b-instruct-q4_K_M', size: '2.0 GB', backend: 'Ollama / Local', status: 'loaded' },
    { name: 'qwen2.5-coder:7b-instruct', size: '4.7 GB', backend: 'Server / RPC Sharded', status: 'available' },
    { name: 'whisper-small.en', size: '480 MB', backend: 'Transcribe Local', status: 'loaded' }
  ]);
</script>

<div class="flex-1 h-screen overflow-y-auto bg-background text-foreground p-6 md:p-8 flex flex-col gap-6 select-none">
  <div>
    <h2 class="text-xl font-bold tracking-tight">Local Models & Inference Router</h2>
    <p class="text-sm text-muted-foreground mt-1">Status of local GGUF models, VRAM memory pools, and Router model dispatch.</p>
  </div>

  <Separator />

  <div class="flex flex-col gap-4">
    {#each models as model}
      <Card.Root>
        <Card.Header class="flex flex-row items-center justify-between gap-2 pb-2">
          <Card.Title class="text-sm font-semibold font-mono">{model.name}</Card.Title>
          <Badge variant={model.status === 'loaded' ? 'default' : 'secondary'} class="text-[10px] uppercase font-mono tracking-wider">
            {model.status}
          </Badge>
        </Card.Header>
        <Card.Content class="pt-0">
          <div class="flex flex-wrap gap-4 text-xs text-muted-foreground">
            <span>Backend: <code class="font-mono text-foreground font-medium">{model.backend}</code></span>
            <span>Memory: <code class="font-mono text-foreground font-medium">{model.size}</code></span>
          </div>
        </Card.Content>
      </Card.Root>
    {/each}
  </div>
</div>
