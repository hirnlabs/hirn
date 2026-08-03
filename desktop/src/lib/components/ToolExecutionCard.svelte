<script lang="ts">
  import type { ToolCall } from '../types/acp';
  import { ChevronDown, Play, Check, X, Circle } from '@lucide/svelte';

  let { toolCall }: { toolCall: ToolCall } = $props();
  let expanded = $state(false);
</script>

<div class="bg-muted/40 border border-border/60 rounded-xl overflow-hidden text-xs my-2 transition-all">
  <button 
    class="w-full flex items-center gap-2.5 px-3.5 py-2.5 bg-muted/60 hover:bg-muted text-left transition-colors font-medium border-none outline-none cursor-pointer"
    onclick={() => expanded = !expanded}
  >
    <span class="flex items-center justify-center shrink-0">
      {#if toolCall.status === 'running'}
        <Play class="size-3.5 text-blue-500 animate-pulse fill-blue-500/20" />
      {:else if toolCall.status === 'completed'}
        <Check class="size-3.5 text-emerald-500 font-bold" />
      {:else if toolCall.status === 'failed'}
        <X class="size-3.5 text-destructive font-bold" />
      {:else}
        <Circle class="size-3.5 text-muted-foreground" />
      {/if}
    </span>
    <span class="font-mono text-[11px] text-foreground/80 flex-1 truncate">{toolCall.name}</span>
    <ChevronDown class="size-3.5 text-muted-foreground/60 transition-transform duration-200 {expanded ? 'rotate-180' : ''}" />
  </button>

  {#if expanded}
    <div class="px-3.5 py-3 bg-muted/20 border-t border-border/40 flex flex-col gap-3.5 animate-in fade-in slide-in-from-top-1 duration-200">
      <div class="flex flex-col gap-1">
        <span class="text-[9px] uppercase tracking-wider font-semibold text-muted-foreground/80 font-sans">Input Parameters</span>
        <pre class="bg-muted/55 border border-border/30 rounded-lg p-2.5 overflow-x-auto text-[10px] font-mono text-foreground/90 max-h-[160px]"><code>{JSON.stringify(toolCall.arguments, null, 2)}</code></pre>
      </div>

      {#if toolCall.result}
        <div class="flex flex-col gap-1">
          <span class="text-[9px] uppercase tracking-wider font-semibold text-muted-foreground/80 font-sans">Execution Result</span>
          <pre class="bg-muted/55 border border-border/30 rounded-lg p-2.5 overflow-x-auto text-[10px] font-mono text-foreground/90 max-h-[200px]"><code>{toolCall.result}</code></pre>
        </div>
      {/if}

      {#if toolCall.error}
        <div class="flex flex-col gap-1">
          <span class="text-[9px] uppercase tracking-wider font-semibold text-destructive/80 font-sans">Error Output</span>
          <pre class="bg-destructive/5 border border-destructive/20 rounded-lg p-2.5 overflow-x-auto text-[10px] font-mono text-destructive max-h-[200px]"><code>{toolCall.error}</code></pre>
        </div>
      {/if}
    </div>
  {/if}
</div>
