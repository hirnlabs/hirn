<script lang="ts">
  export let lang: string = "en";
  export let className: string = "";

  let email = "";
  let isOpen = false;
  let isSubmitted = false;

  // Form selections
  let discovery = "";
  let discoveryOther = "";
  let selectedTools: string[] = [];
  let toolOther = "";

  $: isDe = lang === "de";

  $: labels = {
    placeholder: isDe ? "E-Mail-Adresse eingeben" : "Enter your email",
    buttonText: isDe ? "Zugang erhalten" : "Get early access",
    allSet: isDe ? "Alles erledigt, du hast Zugang!" : "All set, you are on the waitlist!",
    oneMoreThing: isDe ? "Eine Sache noch, bevor du gehst" : "One more thing before you leave",
    q1: isDe ? "Wie hast du von uns erfahren?" : "How did you find us?",
    q2: isDe ? "Welche Tools nutzt du aktuell?" : "What tools are you using right now?",
    otherPlaceholder: isDe ? "Selbst eingeben..." : "Type yourself...",
    submitFeedback: isDe ? "Feedback absenden" : "Submit Feedback",
    thankYou: isDe ? "Vielen Dank für dein Feedback! Wir freuen uns darauf, hirn mit dir zu teilen." : "Thank you for your feedback! We can't wait to share hirn with you.",
    close: isDe ? "Schließen" : "Close",
    otherLabel: isDe ? "Sonstiges" : "Other",
  };

  $: discoveryOptions = isDe
    ? [
        "Web-Suche",
        "Instagram",
        "YouTube",
        "LinkedIn",
        "Jemand hat es mir gezeigt...",
        "Sonstiges",
      ]
    : [
        "Websearch",
        "Instagram",
        "Youtube",
        "Linkedin",
        "Someone showed me...",
        "Other",
      ];

  $: toolOptions = isDe
    ? ["Claude Cowork", "Claude Code", "Codex Desktop", "LmStudio", "Goose", "Sonstiges"]
    : ["Claude Cowork", "Claude Code", "Codex Desktop", "LmStudio", "Goose", "Other"];

  function triggerMailto(emailVal: string, disc?: string, toolsArr?: string[]) {
    const subject = encodeURIComponent(`Early Access signup - hirn (${emailVal})`);
    let bodyText = `Hi,\n\nI'd like to get early access to hirn.\n\nEmail: ${emailVal}`;
    if (disc) {
      bodyText += `\nHow did you find us: ${disc}`;
    }
    if (toolsArr && toolsArr.length > 0) {
      bodyText += `\nTools currently using: ${toolsArr.join(", ")}`;
    }

    const mailtoUrl = `mailto:focccus@proton.me?subject=${subject}&body=${encodeURIComponent(bodyText)}`;
    window.location.href = mailtoUrl;
  }

  function handleSubmitEmail(e: Event) {
    e.preventDefault();
    if (email.trim()) {
      isOpen = true;
      triggerMailto(email);
    }
  }

  function toggleTool(tool: string) {
    if (selectedTools.includes(tool)) {
      selectedTools = selectedTools.filter((t) => t !== tool);
    } else {
      selectedTools = [...selectedTools, tool];
    }
  }

  function isOtherChoice(val: string) {
    return val === "Other" || val === "Sonstiges";
  }

  function handleFinalSubmit(e: Event) {
    e.preventDefault();
    isSubmitted = true;

    const finalDiscovery = isOtherChoice(discovery)
      ? `${discovery}: ${discoveryOther}`
      : discovery;

    const finalTools = selectedTools.map((t) =>
      isOtherChoice(t) ? `${t}: ${toolOther}` : t
    );

    triggerMailto(email, finalDiscovery, finalTools);
  }

  function closeModal() {
    isOpen = false;
    isSubmitted = false;
  }
</script>

<form on:submit={handleSubmitEmail} class={`flex flex-col sm:flex-row gap-3 w-full max-w-[480px] ${className}`}>
  <input
    type="email"
    required
    bind:value={email}
    placeholder={labels.placeholder}
    class="flex-1 px-4 py-[0.8125rem] rounded-xl border border-border bg-surface text-[0.9375rem] text-primary placeholder:text-weak focus:outline-none focus:border-accent focus:ring-2 focus:ring-accent/10 transition-all duration-200"
  />
  <button
    type="submit"
    class="group cursor-pointer rounded-xl bg-accent hover:bg-accent-hover px-6 py-[0.8125rem] text-[0.9375rem] font-semibold whitespace-nowrap text-white dark:text-bg transition-all flex items-center justify-center gap-2 shadow-md"
  >
    {labels.buttonText}
    <svg class="h-4 w-4 transform transition-transform group-hover:translate-x-0.5" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" d="M14 5l7 7m0 0l-7 7m7-7H3"/>
    </svg>
  </button>
</form>

{#if isOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-sm animate-in fade-in duration-200">
    <div class="relative w-full max-w-lg rounded-2xl border border-border bg-surface p-6 sm:p-8 shadow-2xl overflow-hidden max-h-[90vh] overflow-y-auto">
      
      <button
        type="button"
        on:click={closeModal}
        class="absolute top-4 right-4 p-2 rounded-lg text-weak hover:text-primary hover:bg-border/50 transition-colors cursor-pointer"
        aria-label={labels.close}
      >
        <svg class="h-5 w-5" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12"/>
        </svg>
      </button>

      <div class="flex flex-col items-center text-center mb-6">
        <div class="relative flex items-center justify-center h-16 w-16 rounded-full bg-emerald-500/15 text-emerald-500 mb-4 animate-in zoom-in-75 duration-300">
          <svg class="h-10 w-10 stroke-[2.5]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7"/>
          </svg>
        </div>
        <h3 class="text-xl sm:text-2xl font-bold text-primary tracking-tight">
          {labels.allSet}
        </h3>
        <p class="text-sm text-secondary mt-1">
          <span class="font-mono text-accent font-medium">{email}</span>
        </p>
      </div>

      {#if !isSubmitted}
        <form on:submit={handleFinalSubmit} class="space-y-6 pt-4 border-t border-border/60">
          <div class="text-center">
            <h4 class="text-base font-bold text-primary">{labels.oneMoreThing}</h4>
          </div>

          <!-- Q1: Conventional Radio Inputs -->
          <div class="space-y-2.5">
            <label class="block text-xs font-mono font-bold uppercase tracking-wider text-secondary">
              1. {labels.q1}
            </label>
            <div class="grid grid-cols-1 sm:grid-cols-2 gap-2">
              {#each discoveryOptions as opt}
                <label
                  class={`flex items-center gap-3 p-3 rounded-xl border text-xs font-medium cursor-pointer transition-all ${
                    discovery === opt
                      ? "border-accent bg-accent/10 text-primary font-semibold shadow-sm"
                      : "border-border/80 bg-bg/40 text-secondary hover:border-border hover:text-primary"
                  }`}
                >
                  <input
                    type="radio"
                    name="discovery"
                    value={opt}
                    checked={discovery === opt}
                    on:change={() => (discovery = opt)}
                    class="h-4 w-4 accent-accent cursor-pointer"
                  />
                  <span class="flex-1 truncate">{opt}</span>
                </label>
              {/each}
            </div>
            {#if isOtherChoice(discovery)}
              <input
                type="text"
                bind:value={discoveryOther}
                placeholder={labels.otherPlaceholder}
                class="w-full mt-2 px-3.5 py-2.5 text-xs rounded-xl border border-border bg-bg/50 text-primary placeholder:text-weak focus:outline-none focus:border-accent"
              />
            {/if}
          </div>

          <!-- Q2: Conventional Checkbox Inputs -->
          <div class="space-y-2.5">
            <label class="block text-xs font-mono font-bold uppercase tracking-wider text-secondary">
              2. {labels.q2}
            </label>
            <div class="grid grid-cols-1 sm:grid-cols-2 gap-2">
              {#each toolOptions as tool}
                <label
                  class={`flex items-center gap-3 p-3 rounded-xl border text-xs font-medium cursor-pointer transition-all ${
                    selectedTools.includes(tool)
                      ? "border-accent bg-accent/10 text-primary font-semibold shadow-sm"
                      : "border-border/80 bg-bg/40 text-secondary hover:border-border hover:text-primary"
                  }`}
                >
                  <input
                    type="checkbox"
                    name="tools"
                    value={tool}
                    checked={selectedTools.includes(tool)}
                    on:change={() => toggleTool(tool)}
                    class="h-4 w-4 rounded accent-accent cursor-pointer"
                  />
                  <span class="flex-1 truncate">{tool}</span>
                </label>
              {/each}
            </div>
            {#if selectedTools.some(isOtherChoice)}
              <input
                type="text"
                bind:value={toolOther}
                placeholder={labels.otherPlaceholder}
                class="w-full mt-2 px-3.5 py-2.5 text-xs rounded-xl border border-border bg-bg/50 text-primary placeholder:text-weak focus:outline-none focus:border-accent"
              />
            {/if}
          </div>

          <div class="pt-2">
            <button
              type="submit"
              class="w-full cursor-pointer rounded-xl bg-accent hover:bg-accent-hover py-3 text-sm font-bold text-white dark:text-bg transition-all shadow-md flex items-center justify-center gap-2"
            >
              {labels.submitFeedback}
              <svg class="h-4 w-4" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7"/></svg>
            </button>
          </div>
        </form>
      {:else}
        <div class="pt-4 border-t border-border/60 text-center space-y-4 animate-in fade-in duration-300">
          <div class="p-4 rounded-xl bg-emerald-500/10 border border-emerald-500/30 text-emerald-400 text-sm font-semibold">
            {labels.thankYou}
          </div>
          <button
            type="button"
            on:click={closeModal}
            class="px-6 py-2.5 rounded-xl border border-border bg-surface hover:bg-border/60 font-mono text-xs font-bold text-primary transition-colors cursor-pointer"
          >
            {labels.close}
          </button>
        </div>
      {/if}

    </div>
  </div>
{/if}
