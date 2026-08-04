<script lang="ts">
  import type { SessionStore } from '../stores/session.svelte';
  import type { AcpModel } from '../types/acp';
  import ToolExecutionCard from './ToolExecutionCard.svelte';
  import * as InputGroup from './ui/input-group';
  import * as DropdownMenu from './ui/dropdown-menu';
  import { Separator } from './ui/separator';
  import { Input } from './ui/input';
  import { Badge } from './ui/badge';
  import { 
    ChevronDown, 
    Paperclip, 
    X, 
    Search, 
    Star, 
    Check, 
    Compass,
    Wrench,
    Sparkles,
    Play,
    Plus as IconPlus,
    ArrowUp as ArrowUpIcon,
    File,
    Folder,
    Mic,
    MicOff,
    Terminal,
    Globe,
    Code,
    Cpu,
    Layers,
    Bot,
    FileText,
    Info,
    Brain,
    RefreshCw,
    ThumbsUp,
    ThumbsDown,
    GitBranch,
    Repeat
  } from '@lucide/svelte';

  let { store }: { store: SessionStore } = $props();
  let promptText = $state('');

  const recentApps = [
    { name: 'Terminal', icon: Terminal, prompt: 'Run terminal command ' },
    { name: 'Web Search', icon: Globe, prompt: 'Search the web for ' },
    { name: 'Code Editor', icon: Code, prompt: 'Write a program to ' },
    { name: 'Python Exec', icon: Cpu, prompt: 'Analyze dataset with Python ' },
    { name: 'Linear Tasks', icon: Layers, prompt: 'Check my open Linear issues ' }
  ];

  // Slash commands / Skills & Tools grouped into sections
  const slashCommands = [
    {
      group: 'Skills',
      items: [
        { id: 'git_status', name: '/git_status', desc: 'Check git status and modified files', icon: Code },
        { id: 'code_review', name: '/code_review', desc: 'Run automated code review on branch changes', icon: Check },
        { id: 'diagnose', name: '/diagnose', desc: 'Debug issues and inspect system logs', icon: Wrench }
      ]
    },
    {
      group: 'Tools',
      items: [
        { id: 'run_command', name: '/run_command', desc: 'Execute bash/powershell shell command', icon: Terminal },
        { id: 'web_search', name: '/web_search', desc: 'Search the web for info & docs', icon: Globe },
        { id: 'read_file', name: '/read_file', desc: 'Read content from workspace file', icon: File },
        { id: 'linear', name: '/linear', desc: 'Search and sync Linear tasks', icon: Layers }
      ]
    }
  ];

  // Dummy local markdown & project files for @ mention
  const fileMentions = [
    { name: 'CONTEXT-MAP.md', path: 'CONTEXT-MAP.md', desc: 'Domain context map architecture', icon: FileText },
    { name: 'AGENTS.md', path: 'AGENTS.md', desc: 'Agent skills and triage label specs', icon: FileText },
    { name: 'README.md', path: 'README.md', desc: 'Project overview and setup guide', icon: FileText },
    { name: 'docs/domain.md', path: 'docs/domain.md', desc: 'Multi-context domain documentation', icon: FileText },
    { name: 'docs/issue-tracker.md', path: 'docs/issue-tracker.md', desc: 'Linear issue tracker integration', icon: FileText },
    { name: 'src/lib/stores/session.svelte.ts', path: 'src/lib/stores/session.svelte.ts', desc: 'Session state management store', icon: Code }
  ];

  // Autocomplete menu state
  let isSlashMenuOpen = $state(false);
  let isAtMenuOpen = $state(false);
  let menuSearchQuery = $state('');
  let menuSelectedIndex = $state(0);

  type SlashItem = typeof slashCommands[0]['items'][0];

  let groupedSlashCommands = $derived(() => {
    const q = menuSearchQuery.toLowerCase();
    const result: { group: string; items: SlashItem[] }[] = [];
    for (const cat of slashCommands) {
      const filtered = cat.items.filter(c => c.name.toLowerCase().includes(q) || c.desc.toLowerCase().includes(q));
      if (filtered.length > 0) {
        result.push({ group: cat.group, items: filtered });
      }
    }
    return result;
  });

  let flatSlashCommands = $derived(() => {
    return groupedSlashCommands().flatMap(g => g.items);
  });

  let filteredFileMentions = $derived(() => {
    const q = menuSearchQuery.toLowerCase();
    if (!q) return fileMentions;
    return fileMentions.filter(f => f.name.toLowerCase().includes(q) || f.path.toLowerCase().includes(q) || f.desc.toLowerCase().includes(q));
  });



  let isModelMenuOpen = $state(false);
  let isToolsMenuOpen = $state(false);
  let isAttachMenuOpen = $state(false);
  let thinkingEffort = $state<'off' | 'low' | 'mid' | 'high' | 'max'>('mid');
  const thinkingLevels = ['off', 'low', 'mid', 'high', 'max'] as const;
  let thinkingIdx = $derived(thinkingLevels.indexOf(thinkingEffort));
  let searchQuery = $state('');
  let toolsSearchQuery = $state('');
  let editorRef = $state<HTMLDivElement | null>(null);

  function getEditableText(container: HTMLElement | null): string {
    if (!container) return '';
    let result = '';
    for (const node of Array.from(container.childNodes)) {
      if (node.nodeType === Node.TEXT_NODE) {
        result += node.textContent;
      } else if (node.nodeType === Node.ELEMENT_NODE) {
        const el = node as HTMLElement;
        if (el.hasAttribute('data-command')) {
          result += el.getAttribute('data-command');
        } else if (el.hasAttribute('data-mention')) {
          result += el.getAttribute('data-mention');
        } else if (el.tagName === 'BR') {
          result += '\n';
        } else {
          result += el.innerText;
        }
      }
    }
    return result;
  }

  function getTextBeforeCursorInEditable(): string {
    const sel = window.getSelection();
    if (!sel || !sel.rangeCount || !editorRef) return '';
    const range = sel.getRangeAt(0);
    const preRange = range.cloneRange();
    preRange.selectNodeContents(editorRef);
    preRange.setEnd(range.startContainer, range.startOffset);
    return preRange.toString();
  }

  function checkAutocompleteTrigger() {
    const textBeforeCursor = getTextBeforeCursorInEditable();
    if (!textBeforeCursor) {
      isSlashMenuOpen = false;
      isAtMenuOpen = false;
      return;
    }

    const slashMatch = textBeforeCursor.match(/\/([a-zA-Z0-9_-]*)$/);
    if (slashMatch) {
      isSlashMenuOpen = true;
      isAtMenuOpen = false;
      menuSearchQuery = slashMatch[1];
      menuSelectedIndex = 0;
      return;
    }

    const atMatch = textBeforeCursor.match(/@([a-zA-Z0-9_./-]*)$/);
    if (atMatch) {
      isAtMenuOpen = true;
      isSlashMenuOpen = false;
      menuSearchQuery = atMatch[1];
      menuSelectedIndex = 0;
      return;
    }

    isSlashMenuOpen = false;
    isAtMenuOpen = false;
  }

  function handleEditorInput() {
    promptText = getEditableText(editorRef);
    checkAutocompleteTrigger();
  }

  function insertChipNode(type: 'command' | 'mention', text: string) {
    if (!editorRef) return;
    const textBeforeCursor = getTextBeforeCursorInEditable();
    const match = type === 'command'
      ? textBeforeCursor.match(/\/([a-zA-Z0-9_-]*)$/)
      : textBeforeCursor.match(/@([a-zA-Z0-9_./-]*)$/);
    const replaceLen = match ? match[0].length : 0;

    const sel = window.getSelection();
    if (!sel || !sel.rangeCount) return;
    const range = sel.getRangeAt(0);

    if (replaceLen > 0 && range.startContainer.nodeType === Node.TEXT_NODE) {
      const textNode = range.startContainer as Text;
      const offset = range.startOffset;
      const start = Math.max(0, offset - replaceLen);
      textNode.deleteData(start, replaceLen);
      range.setStart(textNode, start);
      range.collapse(true);
    }

    const span = document.createElement('span');
    span.contentEditable = 'false';
    span.className = type === 'command'
      ? 'inline-flex items-center gap-0.5 bg-primary/15 text-primary rounded px-1.5 py-0.5 text-xs font-mono font-semibold mx-0.5 select-none align-baseline'
      : 'inline-flex items-center gap-0.5 bg-primary/15 text-primary rounded px-1.5 py-0.5 text-xs font-mono font-semibold mx-0.5 select-none align-baseline';
    span.setAttribute(type === 'command' ? 'data-command' : 'data-mention', text);

    const label = document.createElement('span');
    label.textContent = text;

    const closeBtn = document.createElement('span');
    closeBtn.textContent = '×';
    closeBtn.className = 'cursor-pointer opacity-60 hover:opacity-100 ml-0.5 text-[10px] leading-none';
    closeBtn.addEventListener('mousedown', (e) => {
      e.preventDefault();
      e.stopPropagation();
      span.nextSibling?.remove(); // remove trailing space
      span.remove();
      promptText = getEditableText(editorRef);
    });

    span.appendChild(label);
    span.appendChild(closeBtn);

    const space = document.createTextNode('\u00A0');

    range.insertNode(space);
    range.insertNode(span);

    range.setStartAfter(space);
    range.setEndAfter(space);
    sel.removeAllRanges();
    sel.addRange(range);

    promptText = getEditableText(editorRef);
    isSlashMenuOpen = false;
    isAtMenuOpen = false;
  }

  function selectSlashCommand(cmd: SlashItem) {
    insertChipNode('command', cmd.name);
  }

  function selectFileMention(file: typeof fileMentions[0]) {
    insertChipNode('mention', `@${file.name}`);
  }

  // Voice Input & Recording State
  let isRecording = $state(false);
  let recordingBarHeights = $state<number[]>([4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4]);
  let recordingInterval = $state<any>(null);
  let speechRecognition = $state<any>(null);
  let audioContext = $state<AudioContext | null>(null);
  let mediaStream = $state<MediaStream | null>(null);
  let animFrameId = $state<number | null>(null);

  function toggleVoiceRecording() {
    if (isRecording) {
      stopVoiceRecording();
    } else {
      startVoiceRecording();
    }
  }

  async function startVoiceRecording() {
    isRecording = true;

    // Focus editor and insert dictation text node at current cursor position
    if (editorRef) {
      editorRef.focus();
    }

    let dictationNode: Text | null = null;
    const sel = window.getSelection();
    if (sel && sel.rangeCount > 0 && editorRef && editorRef.contains(sel.getRangeAt(0).commonAncestorContainer)) {
      const range = sel.getRangeAt(0);
      range.deleteContents();
      dictationNode = document.createTextNode('');
      range.insertNode(dictationNode);
    } else if (editorRef) {
      dictationNode = document.createTextNode('');
      editorRef.appendChild(dictationNode);
    }

    // Web Audio API real-time microphone analyzer
    if (typeof window !== 'undefined' && navigator.mediaDevices) {
      try {
        const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
        mediaStream = stream;

        const AudioCtx = window.AudioContext || (window as any).webkitAudioContext;
        const ctx = new AudioCtx();
        audioContext = ctx;

        const source = ctx.createMediaStreamSource(stream);
        const analyser = ctx.createAnalyser();
        analyser.fftSize = 64;
        analyser.smoothingTimeConstant = 0.75;
        source.connect(analyser);

        const bufferLength = analyser.frequencyBinCount;
        const dataArray = new Uint8Array(bufferLength);

        const updateLiveBars = () => {
          if (!isRecording) return;
          analyser.getByteFrequencyData(dataArray);

          const numBars = 12;
          const step = Math.max(1, Math.floor(bufferLength / numBars));
          const updated: number[] = [];

          for (let i = 0; i < numBars; i++) {
            const val = dataArray[i * step] || 0;
            // Map 0-255 frequency level to bar height 4px - 20px
            const barHeight = Math.max(4, Math.floor((val / 255) * 16 + 4));
            updated.push(barHeight);
          }

          recordingBarHeights = updated;
          animFrameId = requestAnimationFrame(updateLiveBars);
        };

        updateLiveBars();
      } catch (err) {
        console.warn('Microphone access unavailable, using animated fallback:', err);
        if (recordingInterval) clearInterval(recordingInterval);
        recordingInterval = setInterval(() => {
          recordingBarHeights = recordingBarHeights.map(() => Math.floor(Math.random() * 14) + 4);
        }, 90);
      }
    }

    // Web Speech API live transcription
    if (typeof window !== 'undefined') {
      const SpeechRecognition = (window as any).SpeechRecognition || (window as any).webkitSpeechRecognition;
      if (SpeechRecognition) {
        try {
          const rec = new SpeechRecognition();
          rec.continuous = true;
          rec.interimResults = true;
          rec.lang = 'en-US';

          let sessionFinalText = '';

          rec.onresult = (event: any) => {
            let interimTranscript = '';
            let finalChunk = '';
            for (let i = event.resultIndex; i < event.results.length; i++) {
              if (event.results[i].isFinal) {
                finalChunk += event.results[i][0].transcript;
              } else {
                interimTranscript += event.results[i][0].transcript;
              }
            }
            if (finalChunk) {
              sessionFinalText += finalChunk;
            }

            const currentTranscript = sessionFinalText + interimTranscript;
            if (currentTranscript && dictationNode) {
              dictationNode.nodeValue = currentTranscript;
              promptText = getEditableText(editorRef);

              if (editorRef) {
                const currentSel = window.getSelection();
                if (currentSel) {
                  const range = document.createRange();
                  range.setStartAfter(dictationNode);
                  range.collapse(true);
                  currentSel.removeAllRanges();
                  currentSel.addRange(range);
                }
              }
            }
          };

          rec.onerror = (err: any) => {
            console.warn('Speech recognition error:', err);
          };

          rec.onend = () => {
            if (isRecording) {
              if (dictationNode && dictationNode.nodeValue) {
                sessionFinalText = '';
                const newRange = document.createRange();
                newRange.setStartAfter(dictationNode);
                newRange.collapse(true);
                dictationNode = document.createTextNode('');
                newRange.insertNode(dictationNode);
              }
              try { rec.start(); } catch (e) {}
            }
          };

          rec.start();
          speechRecognition = rec;
        } catch (e) {
          console.warn('Could not start SpeechRecognition:', e);
        }
      }
    }
  }

  function stopVoiceRecording() {
    isRecording = false;

    if (animFrameId) {
      cancelAnimationFrame(animFrameId);
      animFrameId = null;
    }
    if (mediaStream) {
      mediaStream.getTracks().forEach(track => track.stop());
      mediaStream = null;
    }
    if (audioContext) {
      audioContext.close();
      audioContext = null;
    }
    if (recordingInterval) {
      clearInterval(recordingInterval);
      recordingInterval = null;
    }
    if (speechRecognition) {
      try {
        speechRecognition.stop();
      } catch (e) {}
      speechRecognition = null;
    }
    recordingBarHeights = [4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4];
  }

  // File & Folder Attachments State
  let fileInputRef = $state<HTMLInputElement | null>(null);
  let folderInputRef = $state<HTMLInputElement | null>(null);
  let attachedFiles = $state<{ name: string; path: string; isFolder?: boolean }[]>([]);

  let activeSession = $derived(store.activeSession);

  // Local Storage Persistence for Favorites and Selected Model
  let favoriteModelIds = $state<string[]>(loadFavorites());

  // 3-State & 4-State Permission Control System
  export type PermissionState = 'off' | 'ask' | 'allow';
  export type ParentPermissionState = PermissionState | 'per_tool';

  interface SubToolItem {
    id: string;
    name: string;
    permission: PermissionState;
  }

  interface ExtensionItem {
    id: string;
    name: string;
    icon: string;
    section: 'Built-In Tools' | 'Apps' | 'Skills' | 'Recipes';
    permission: ParentPermissionState;
    tools?: SubToolItem[];
  }

  let extensions = $state<ExtensionItem[]>(loadExtensions());
  let selectedExtForTools = $state<ExtensionItem | null>(null);

  function loadFavorites(): string[] {
    try {
      const stored = localStorage.getItem('hirn_favorite_models');
      return stored ? JSON.parse(stored) : ['local/gemma-4-9b-it', 'anthropic/claude-3-5-sonnet'];
    } catch {
      return ['local/gemma-4-9b-it', 'anthropic/claude-3-5-sonnet'];
    }
  }

  function loadExtensions(): ExtensionItem[] {
    const defaults: ExtensionItem[] = [
      // Apps Section
      { id: 'ext-apps', name: 'Apps', icon: '📦', section: 'Apps', permission: 'allow', tools: [{ id: 'app_sandbox', name: 'App Sandbox', permission: 'allow' }] },
      { id: 'ext-dev', name: 'Developer', icon: '⚡', section: 'Apps', permission: 'per_tool', tools: [{ id: 'read_file', name: 'Read File', permission: 'allow' }, { id: 'write_file', name: 'Write File', permission: 'ask' }, { id: 'run_cmd', name: 'Run Command', permission: 'ask' }] },
      { id: 'ext-mgr', name: 'Extension Manager', icon: '🧩', section: 'Apps', permission: 'allow' },
      { id: 'ext-todo', name: 'Todo', icon: '☑️', section: 'Apps', permission: 'allow', tools: [{ id: 'manage_task', name: 'Manage Task', permission: 'allow' }] },
      { id: 'ext-visualiser', name: 'Autovisualiser', icon: '🎨', section: 'Apps', permission: 'allow', tools: [{ id: 'generate_image', name: 'Generate Image', permission: 'allow' }] },

      // Skills Section
      { id: 'ext-analyze', name: 'Analyze', icon: '🔍', section: 'Skills', permission: 'allow', tools: [{ id: 'grep_search', name: 'Grep Search', permission: 'allow' }, { id: 'code_survey', name: 'Code Survey', permission: 'allow' }] },
      { id: 'ext-skills', name: 'Skills', icon: '🎯', section: 'Skills', permission: 'allow', tools: [{ id: 'linear_mcp', name: 'Linear MCP', permission: 'allow' }] },

      // Recipes Section
      { id: 'ext-summon', name: 'Summon', icon: '🔮', section: 'Recipes', permission: 'ask', tools: [{ id: 'subagent', name: 'Subagent Dispatch', permission: 'ask' }] },
      { id: 'ext-tom', name: 'Tom', icon: '👤', section: 'Recipes', permission: 'allow' },
      { id: 'ext-tutorial', name: 'Tutorial', icon: '📚', section: 'Recipes', permission: 'allow' },

      // Built-In Tools Section
      { id: 'ext-builtin-read', name: 'Read File', icon: '📄', section: 'Built-In Tools', permission: 'allow' },
      { id: 'ext-builtin-write', name: 'Write File', icon: '✏️', section: 'Built-In Tools', permission: 'ask' },
      { id: 'ext-builtin-cmd', name: 'Run Command', icon: '🐚', section: 'Built-In Tools', permission: 'ask' },
      { id: 'ext-builtin-search', name: 'Web Search', icon: '🌐', section: 'Built-In Tools', permission: 'allow' },
      { id: 'ext-builtin-agent', name: 'Subagent Manager', icon: '🤖', section: 'Built-In Tools', permission: 'ask' }
    ];

    try {
      const stored = localStorage.getItem('hirn_session_extensions_v2');
      if (stored) {
        const parsed = JSON.parse(stored);
        if (Array.isArray(parsed) && parsed.length > 0 && parsed.every(item => item.permission)) {
          return parsed;
        }
      }
    } catch (e) {
      console.warn('Failed to load extensions from localStorage', e);
    }
    return defaults;
  }

  function saveExtensions() {
    try {
      localStorage.setItem('hirn_session_extensions_v2', JSON.stringify(extensions));
    } catch (e) {
      console.warn('Failed to save extensions to localStorage', e);
    }
  }

  function setParentPermission(extId: string, perm: ParentPermissionState) {
    const ext = extensions.find(e => e.id === extId);
    if (!ext) return;

    ext.permission = perm;

    if (perm !== 'per_tool' && ext.tools) {
      ext.tools.forEach(t => t.permission = perm as PermissionState);
    } else if (perm === 'per_tool' && ext.tools && ext.tools.length > 0) {
      selectedExtForTools = ext;
    }

    saveExtensions();
  }

  function setToolPermission(extId: string, toolId: string, perm: PermissionState) {
    const ext = extensions.find(e => e.id === extId);
    if (!ext || !ext.tools) return;

    const tool = ext.tools.find(t => t.id === toolId);
    if (!tool) return;

    tool.permission = perm;

    const allSame = ext.tools.every(t => t.permission === tool.permission);
    if (allSame) {
      ext.permission = tool.permission;
    } else {
      ext.permission = 'per_tool';
    }

    saveExtensions();
  }

  function handleExtensionRowClick(ext: ExtensionItem) {
    if (ext.tools && ext.tools.length > 0) {
      selectedExtForTools = selectedExtForTools?.id === ext.id ? null : ext;
    }
  }

  // Unified Attachment Menu Triggers
  function toggleAttachMenu(e: MouseEvent) {
    e.stopPropagation();
    isAttachMenuOpen = !isAttachMenuOpen;
    isModelMenuOpen = false;
    isToolsMenuOpen = false;
  }

  function handleAttachFiles() {
    isAttachMenuOpen = false;
    fileInputRef?.click();
  }

  function handleAttachFolder() {
    isAttachMenuOpen = false;
    folderInputRef?.click();
  }

  function handleFileInputChange(e: Event) {
    const target = e.target as HTMLInputElement;
    if (target.files) {
      const files = Array.from(target.files).map(f => ({
        name: f.name,
        path: f.name,
        isFolder: false
      }));
      attachedFiles = [...attachedFiles, ...files];
    }
  }

  function handleFolderInputChange(e: Event) {
    const target = e.target as HTMLInputElement;
    if (target.files && target.files.length > 0) {
      const firstFile = target.files[0];
      const folderName = firstFile.webkitRelativePath.split('/')[0] || firstFile.name;
      attachedFiles = [...attachedFiles, {
        name: folderName,
        path: folderName,
        isFolder: true
      }];
    }
  }

  function removeFile(index: number) {
    attachedFiles = attachedFiles.filter((_, i) => i !== index);
  }

  let totalActiveToolsCount = $derived(() => {
    let count = 0;
    for (const ext of extensions) {
      if (ext.tools && ext.tools.length > 0) {
        count += ext.tools.filter(t => t.permission !== 'off').length;
      } else if (ext.permission !== 'off') {
        count += 1;
      }
    }
    return count;
  });

  let filteredExtensions = $derived(() => {
    const query = toolsSearchQuery.trim().toLowerCase();
    if (!query) return extensions;
    return extensions.filter(e =>
      e.name.toLowerCase().includes(query) ||
      (e.tools && e.tools.some(t => t.name.toLowerCase().includes(query)))
    );
  });

  let groupedExtensions = $derived(() => {
    const all = filteredExtensions();
    const sections: { header: string; items: ExtensionItem[] }[] = [
      { header: 'Apps', items: all.filter(e => e.section === 'Apps') },
      { header: 'Skills', items: all.filter(e => e.section === 'Skills') },
      { header: 'Recipes', items: all.filter(e => e.section === 'Recipes') },
      { header: 'Built-In Tools', items: all.filter(e => e.section === 'Built-In Tools') }
    ];
    return sections.filter(s => s.items.length > 0);
  });

  function saveFavorites(favs: string[]) {
    favoriteModelIds = favs;
    try {
      localStorage.setItem('hirn_favorite_models', JSON.stringify(favs));
    } catch (e) {
      console.warn('Failed to save favorites to localStorage', e);
    }
  }

  function selectModel(modelId: string) {
    if (activeSession) {
      activeSession.selectedModelId = modelId;
      try {
        localStorage.setItem('hirn_selected_model', modelId);
      } catch (e) {
        console.warn('Failed to save selected model to localStorage', e);
      }
    }
    isModelMenuOpen = false;
  }

  function toggleFavorite(event: Event, modelId: string) {
    event.stopPropagation();
    if (favoriteModelIds.includes(modelId)) {
      saveFavorites(favoriteModelIds.filter(id => id !== modelId));
    } else {
      saveFavorites([...favoriteModelIds, modelId]);
    }
  }

  let selectedModel = $derived(
    activeSession?.availableModels.find(m => m.id === activeSession.selectedModelId) ?? activeSession?.availableModels[0]
  );

  let filteredModels = $derived(() => {
    if (!activeSession) return [];
    const query = searchQuery.trim().toLowerCase();
    if (!query) return activeSession.availableModels;
    return activeSession.availableModels.filter(
      m => m.name.toLowerCase().includes(query) || m.provider.toLowerCase().includes(query)
    );
  });

  let groupedModels = $derived(() => {
    const all = filteredModels();
    const favorites = all.filter(m => favoriteModelIds.includes(m.id));
    const regularMap = new Map<string, AcpModel[]>();

    for (const m of all) {
      const list = regularMap.get(m.provider) || [];
      list.push(m);
      regularMap.set(m.provider, list);
    }

    const groups: { header: string; models: AcpModel[] }[] = [];

    if (favorites.length > 0 && !searchQuery) {
      groups.push({ header: 'Favorite', models: favorites });
    }

    for (const [provider, models] of regularMap.entries()) {
      groups.push({ header: provider, models });
    }

    return groups;
  });

  function focusPromptWindowAtStart() {
    if (!editorRef) return;
    editorRef.focus();
    const sel = window.getSelection();
    if (!sel) return;
    const range = document.createRange();
    if (editorRef.firstChild) {
      range.setStart(editorRef.firstChild, 0);
    } else {
      range.setStart(editorRef, 0);
    }
    range.collapse(true);
    sel.removeAllRanges();
    sel.addRange(range);
  }

  let currentActiveSessionId = $state<string | null>(null);

  $effect(() => {
    const sid = activeSession?.id ?? null;
    if (sid && sid !== currentActiveSessionId) {
      currentActiveSessionId = sid;
      promptText = '';
      if (editorRef) editorRef.innerHTML = '';
      requestAnimationFrame(() => {
        focusPromptWindowAtStart();
      });
    }
  });

  $effect(() => {
    if (store.autoStartRecording && activeSession) {
      store.autoStartRecording = false;
      startVoiceRecording();
    }
  });

  function handleSubmit() {
    const text = getEditableText(editorRef).trim();
    if ((!text && attachedFiles.length === 0) || !activeSession) return;
    if (isRecording) {
      stopVoiceRecording();
    }
    let messageText = text;
    if (attachedFiles.length > 0) {
      const fileListStr = attachedFiles.map(f => f.isFolder ? `[Attached Folder: ${f.name}](${f.path})` : `[Attached File: ${f.name}](${f.path})`).join('\n');
      messageText = messageText ? `${messageText}\n\n${fileListStr}` : fileListStr;
    }
    store.sendMessage(activeSession.id, messageText);
    promptText = '';
    if (editorRef) editorRef.innerHTML = '';
    attachedFiles = [];
  }

  function scrollSelectedIntoView() {
    requestAnimationFrame(() => {
      const el = document.querySelector('[data-menu-selected="true"]') as HTMLElement | null;
      el?.scrollIntoView({ block: 'nearest' });
    });
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (isSlashMenuOpen) {
      const items = flatSlashCommands();
      if (event.key === 'ArrowDown') {
        event.preventDefault();
        menuSelectedIndex = (menuSelectedIndex + 1) % Math.max(items.length, 1);
        scrollSelectedIntoView();
        return;
      }
      if (event.key === 'ArrowUp') {
        event.preventDefault();
        menuSelectedIndex = (menuSelectedIndex - 1 + items.length) % Math.max(items.length, 1);
        scrollSelectedIntoView();
        return;
      }
      if ((event.key === 'Enter' || event.key === 'Tab') && items[menuSelectedIndex]) {
        event.preventDefault();
        selectSlashCommand(items[menuSelectedIndex]);
        return;
      }
      if (event.key === 'Escape') {
        event.preventDefault();
        isSlashMenuOpen = false;
        return;
      }
    }

    if (isAtMenuOpen) {
      const items = filteredFileMentions();
      if (event.key === 'ArrowDown') {
        event.preventDefault();
        menuSelectedIndex = (menuSelectedIndex + 1) % Math.max(items.length, 1);
        scrollSelectedIntoView();
        return;
      }
      if (event.key === 'ArrowUp') {
        event.preventDefault();
        menuSelectedIndex = (menuSelectedIndex - 1 + items.length) % Math.max(items.length, 1);
        scrollSelectedIntoView();
        return;
      }
      if ((event.key === 'Enter' || event.key === 'Tab') && items[menuSelectedIndex]) {
        event.preventDefault();
        selectFileMention(items[menuSelectedIndex]);
        return;
      }
      if (event.key === 'Escape') {
        event.preventDefault();
        isAtMenuOpen = false;
        return;
      }
    }

    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault();
      handleSubmit();
    }
  }

  function handleBackdropClick(e: MouseEvent) {
    const target = e.target as HTMLElement;
    if (!target.closest('.model-picker-container')) {
      isModelMenuOpen = false;
    }
    if (!target.closest('.tools-picker-container')) {
      isToolsMenuOpen = false;
    }
    if (!target.closest('.attach-picker-container')) {
      isAttachMenuOpen = false;
    }
  }
</script>

<svelte:window onclick={handleBackdropClick} />

<div class="flex-1 flex flex-col h-screen bg-background text-foreground overflow-x-hidden relative">
  {#if activeSession}
    <!-- Header with centered Model picker (LibreChat Style) -->
    <header class="flex items-center justify-between px-6 py-3 border-b border-border/60 bg-background relative z-40">
      <div class="flex flex-col min-w-0">
        <h2 class="text-sm font-bold text-foreground truncate max-w-[200px] sm:max-w-xs">{activeSession.title}</h2>
        <div class="flex items-center gap-1.5 mt-0.5">
          <span class="text-[10px] text-muted-foreground bg-muted px-1.5 py-0.5 rounded font-mono border border-border/30">{activeSession.agentName}</span>
        </div>
      </div>

      <div class="flex items-center gap-3">
        <span class="text-[10px] text-muted-foreground bg-muted px-2 py-1 rounded border border-border/30 font-medium">
          {activeSession.status}
        </span>
      </div>
    </header>

    {#if activeSession.messages.length > 0}
      <!-- Chat Messages Feed -->
      <div class="flex-1 overflow-y-auto px-4 py-6 md:px-8 space-y-6 scrollbar-thin">
        <div class="max-w-[768px] mx-auto space-y-6">
          {#each activeSession.messages as msg (msg.id)}
            {@const usedCtx = msg.contextTokens ?? 6144}
            {@const maxCtx = msg.maxContextTokens ?? selectedModel?.contextWindow ?? 128000}
            {@const pctCtx = Math.min(100, Math.round((usedCtx / maxCtx) * 1000) / 10)}
            {@const usedStr = usedCtx >= 1000 ? `${(usedCtx / 1000).toFixed(1)}k` : `${usedCtx}`}
            {@const maxStr = maxCtx >= 1000 ? `${(maxCtx / 1000).toFixed(0)}k` : `${maxCtx}`}
            <div class="flex flex-col gap-2">
              {#if msg.role === 'user'}
                <!-- User message: rounded card bubble -->
                <div class="flex justify-end">
                  <div class="bg-muted rounded-2xl px-4 py-2.5 text-sm text-foreground max-w-[85%] whitespace-pre-wrap">
                    {msg.content}
                  </div>
                </div>
              {:else}
                <!-- Assistant message: clean markdown structure with details for thoughts -->
                <div class="flex flex-col gap-3">
                  {#if msg.thoughts}
                    <details class="bg-muted/40 rounded-xl p-3 text-xs text-muted-foreground group">
                      <summary class="font-semibold text-foreground/80 cursor-pointer list-none flex items-center justify-between">
                        <span>Thought Process</span>
                        <ChevronDown class="size-3.5 text-gray-500 transition-transform group-open:rotate-180" />
                      </summary>
                      <p class="mt-2 leading-relaxed whitespace-pre-wrap pt-2">{msg.thoughts}</p>
                    </details>
                  {/if}

                  {#if msg.toolCalls}
                    <div class="space-y-2">
                      {#each msg.toolCalls as tool}
                        <ToolExecutionCard toolCall={tool} />
                      {/each}
                    </div>
                  {/if}

                  {#if msg.content}
                    <div class="text-sm text-foreground/90 leading-relaxed whitespace-pre-wrap select-text">
                      {msg.content}
                    </div>
                  {/if}

                  <!-- Response actions -->
                  <div class="flex items-center gap-0.5 mt-1.5 select-none">
                    <!-- Diagnostics info icon with hover popover (seamless hover bridge & expanded card design) -->
                    <div class="relative group/info">
                      <button
                        type="button"
                        class="p-1 rounded-md text-muted-foreground/40 hover:text-muted-foreground hover:bg-muted/60 transition-colors"
                        title="Response diagnostics"
                      >
                        <Info class="size-3.5" />
                      </button>
                      <div
                        class="absolute bottom-full left-0 mb-0 pb-2 opacity-0 pointer-events-none group-hover/info:opacity-100 group-hover/info:pointer-events-auto transition-opacity duration-150 z-50"
                      >
                        <div class="w-64 p-3 rounded-2xl bg-popover border border-border shadow-2xl text-xs flex flex-col gap-2 font-sans select-text">
                          <div class="flex items-center justify-between border-b border-border/50 pb-2">
                            <span class="font-semibold text-foreground truncate max-w-[150px]">{selectedModel?.name ?? 'model'}</span>
                            <span class="text-[10px] text-muted-foreground bg-muted px-1.5 py-0.5 rounded font-mono border border-border/30 shrink-0">{selectedModel?.provider ?? 'Local'}</span>
                          </div>

                          <div class="grid grid-cols-2 gap-2 text-[11px] font-mono">
                            <div class="flex flex-col bg-muted/40 p-2 rounded-xl border border-border/30">
                              <span class="text-[9px] text-muted-foreground uppercase font-sans font-medium">Speed</span>
                              <span class="text-foreground font-semibold">{msg.tokensPerSec ?? 42} tok/s</span>
                            </div>
                            <div class="flex flex-col bg-muted/40 p-2 rounded-xl border border-border/30">
                              <span class="text-[9px] text-muted-foreground uppercase font-sans font-medium">Latency</span>
                              <span class="text-foreground font-semibold">{msg.latencyMs ?? 320} ms</span>
                            </div>
                            <div class="flex flex-col bg-muted/40 p-2 rounded-xl border border-border/30 col-span-2">
                              <div class="flex items-center justify-between">
                                <span class="text-[9px] text-muted-foreground uppercase font-sans font-medium">Context Window</span>
                                <span class="text-foreground font-semibold font-mono text-[10px]">{usedStr} / {maxStr} ({pctCtx}%)</span>
                              </div>
                              <div class="w-full h-1.5 rounded-full bg-muted mt-1.5 overflow-hidden">
                                <div class="h-full bg-primary rounded-full transition-all" style="width: {pctCtx}%"></div>
                              </div>
                            </div>
                          </div>
                        </div>
                      </div>
                    </div>

                    <!-- Refresh / Regenerate -->
                    <button
                      type="button"
                      class="p-1 rounded-md text-muted-foreground/40 hover:text-muted-foreground hover:bg-muted/60 transition-colors"
                      title="Regenerate response"
                    >
                      <RefreshCw class="size-3.5" />
                    </button>

                    <!-- Thumbs up -->
                    <button
                      type="button"
                      class="p-1 rounded-md text-muted-foreground/40 hover:text-muted-foreground hover:bg-muted/60 transition-colors"
                      title="Good response"
                    >
                      <ThumbsUp class="size-3.5" />
                    </button>

                    <!-- Thumbs down -->
                    <button
                      type="button"
                      class="p-1 rounded-md text-muted-foreground/40 hover:text-muted-foreground hover:bg-muted/60 transition-colors"
                      title="Bad response"
                    >
                      <ThumbsDown class="size-3.5" />
                    </button>

                    <!-- Branch conversation -->
                    <button
                      type="button"
                      class="p-1 rounded-md text-muted-foreground/40 hover:text-muted-foreground hover:bg-muted/60 transition-colors"
                      title="Branch from here"
                    >
                      <GitBranch class="size-3.5" />
                    </button>

                    <!-- Multi-model switch -->
                    <button
                      type="button"
                      class="p-1 rounded-md text-muted-foreground/40 hover:text-muted-foreground hover:bg-muted/60 transition-colors"
                      title="Re-run with different model"
                    >
                      <Repeat class="size-3.5" />
                    </button>
                  </div>
                </div>
              {/if}
            </div>
          {/each}
        </div>
      </div>

      <!-- Floating Input Bar at Bottom (when messages exist) -->
      <div class="p-4 md:pb-6 bg-background">
        <div class="max-w-[768px] mx-auto w-full relative">
          {@render promptWindow()}
        </div>
      </div>
    {:else}
      <!-- Redesigned Empty Chat View: Centered Prompt Window + Recently Used App Icons -->
      <div class="flex-1 flex flex-col items-center justify-center p-4 md:p-6 bg-background">
        <div class="max-w-[768px] w-full flex flex-col items-center gap-6">
          <div class="w-full relative">
            {@render promptWindow()}
          </div>

          <!-- Recently Used Apps / Tools Row Underneath (Chrome New-Tab Style Rounded Squares) -->
          <div class="flex flex-col items-center gap-3 w-full mt-4">
            <div class="flex items-center justify-center flex-wrap gap-6">
              {#each recentApps as app}
                <button
                  onclick={() => {
                    promptText = app.prompt;
                    if (editorRef) {
                      editorRef.innerText = app.prompt;
                      editorRef.focus();
                    }
                  }}
                  class="flex flex-col items-center gap-2 group cursor-pointer transition-all"
                  title="Use {app.name}"
                >
                  <div class="size-12 rounded-2xl bg-muted/50 hover:bg-muted border border-border/40 group-hover:border-border flex items-center justify-center shadow-sm group-hover:shadow-md group-hover:scale-105 active:scale-95 transition-all">
                    <app.icon class="size-5 text-foreground/80 group-hover:text-primary transition-colors shrink-0" />
                  </div>
                  <span class="text-[11px] font-medium text-muted-foreground group-hover:text-foreground transition-colors max-w-[80px] truncate text-center">
                    {app.name}
                  </span>
                </button>
              {/each}
            </div>
          </div>
        </div>
      </div>
    {/if}

    <!-- Hidden HTML input elements -->
    <input
      type="file"
      multiple
      bind:this={fileInputRef}
      onchange={handleFileInputChange}
      style="display: none;"
    />
    <input
      type="file"
      webkitdirectory
      bind:this={folderInputRef}
      onchange={handleFolderInputChange}
      style="display: none;"
    />
  {:else}
    <div class="flex-1 flex flex-col items-center justify-center text-center gap-3">
      <Compass class="size-10 text-gray-500 animate-spin" />
      <p class="text-sm text-gray-400">No active session selected. Create or select a session to begin.</p>
    </div>
  {/if}
</div>

{#snippet promptWindow()}
  <InputGroup.Root class="!opacity-100 !bg-muted/50 border !border-transparent focus-within:!border-border rounded-[24px] shadow-md focus-within:shadow-lg transition-all p-3 flex flex-col gap-2.5 relative group/prompt">
    <!-- Autocomplete Popover for / Slash Commands (Grouped Sections, compact max-h-[190px]) -->
    {#if isSlashMenuOpen && flatSlashCommands().length > 0}
      <div class="absolute bottom-full left-3 mb-2 w-[320px] bg-popover border border-border text-popover-foreground rounded-2xl shadow-2xl z-50 overflow-hidden flex flex-col p-1.5 animate-in fade-in duration-100 max-h-[190px] overflow-y-auto">
        {#each groupedSlashCommands() as cat}
          <div class="flex flex-col gap-0.5 mb-1">
            <div class="text-[9px] font-semibold text-muted-foreground uppercase tracking-wider px-2 py-0.5 font-mono">
              {cat.group}
            </div>
            {#each cat.items as cmd}
              {@const globalIdx = flatSlashCommands().findIndex(c => c.id === cmd.id)}
              <button
                onclick={() => selectSlashCommand(cmd)}
                data-menu-selected={globalIdx === menuSelectedIndex ? 'true' : undefined}
                class="flex items-center gap-2.5 px-2.5 py-1.5 rounded-xl text-xs text-left transition-all cursor-pointer {globalIdx === menuSelectedIndex ? 'bg-accent text-accent-foreground' : 'hover:bg-accent/60 text-foreground/80'}"
              >
                <div class="size-6 rounded-lg bg-muted flex items-center justify-center shrink-0 border border-border/40">
                  <cmd.icon class="size-3.5 text-primary shrink-0" />
                </div>
                <div class="flex flex-col min-w-0 flex-1">
                  <span class="font-mono font-semibold text-foreground text-[11px]">{cmd.name}</span>
                  <span class="text-[10px] text-muted-foreground truncate">{cmd.desc}</span>
                </div>
              </button>
            {/each}
          </div>
        {/each}
      </div>
    {/if}

    <!-- Autocomplete Popover for @ File Mentions (Compact max-h-[190px]) -->
    {#if isAtMenuOpen && filteredFileMentions().length > 0}
      <div class="absolute bottom-full left-3 mb-2 w-[320px] bg-popover border border-border text-popover-foreground rounded-2xl shadow-2xl z-50 overflow-hidden flex flex-col p-1.5 animate-in fade-in duration-100 max-h-[190px] overflow-y-auto">
        <div class="text-[9px] font-semibold text-muted-foreground uppercase tracking-wider px-2 py-0.5 font-mono border-b border-border/40 mb-1 flex items-center justify-between">
          <span>Files</span>
          <span class="font-mono text-[9px]">{filteredFileMentions().length} files</span>
        </div>
        {#each filteredFileMentions() as file, i (file.path)}
          <button
            onclick={() => selectFileMention(file)}
            data-menu-selected={i === menuSelectedIndex ? 'true' : undefined}
            class="flex items-center gap-2.5 px-2.5 py-1.5 rounded-xl text-xs text-left transition-all cursor-pointer {i === menuSelectedIndex ? 'bg-accent text-accent-foreground' : 'hover:bg-accent/60 text-foreground/80'}"
          >
            <div class="size-6 rounded-lg bg-muted flex items-center justify-center shrink-0 border border-border/40">
              <file.icon class="size-3.5 text-primary shrink-0" />
            </div>
            <div class="flex flex-col min-w-0 flex-1">
              <span class="font-mono font-semibold text-foreground text-[11px] truncate">@{file.name}</span>
              <span class="text-[10px] text-muted-foreground truncate">{file.desc}</span>
            </div>
          </button>
        {/each}
      </div>
    {/if}

    <!-- Attached Files Pills -->
    {#if attachedFiles.length > 0}
      <div class="flex flex-wrap gap-1.5 px-2">
        {#each attachedFiles as file, idx (idx)}
          <div class="flex items-center gap-1.5 bg-background border border-border rounded-xl px-2.5 py-1 text-xs text-foreground shadow-sm">
            <span>{file.isFolder ? '📁' : '📎'}</span>
            <span class="max-w-[150px] truncate font-medium">{file.name}</span>
            <button onclick={() => removeFile(idx)} class="text-gray-500 hover:text-white transition-colors">
              <X class="size-3.5" />
            </button>
          </div>
        {/each}
      </div>
    {/if}

    <!-- Native contenteditable rich prompt editor -->
    <div class="relative w-full min-h-[36px]">
      {#if !promptText || !promptText.trim()}
        <span class="absolute left-3 top-1 text-muted-foreground/60 pointer-events-none text-sm select-none">
          Ask, Search or Chat...
        </span>
      {/if}
      <div
        bind:this={editorRef}
        contenteditable="true"
        role="textbox"
        tabindex="0"
        oninput={handleEditorInput}
        onkeydown={handleKeyDown}
        class="w-full !bg-transparent border-none outline-none text-sm text-foreground py-1 px-3 min-h-[36px] max-h-[200px] overflow-y-auto leading-relaxed select-text font-sans focus:outline-none focus:ring-0 [&:focus]:outline-none"
        style="outline: none;"
      ></div>
    </div>

    <!-- Input bar tools -->
    <InputGroup.Addon align="block-end" class="flex items-center border-t border-border/40 pt-2 px-1 relative">
      <!-- Centered Voice Input Button & 3x Expanded Recording Pill (Aligned with bottom bar icons, original size-11) -->
      <div class="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 pointer-events-auto z-10">
        {#if !isRecording}
          <button
            onclick={toggleVoiceRecording}
            class="size-11 rounded-full bg-primary text-primary-foreground hover:bg-primary/90 shadow-md hover:shadow-lg hover:scale-105 active:scale-95 transition-all duration-200 cursor-pointer flex items-center justify-center font-bold group/voicebtn {promptText.trim() ? 'opacity-40 hover:opacity-100' : 'opacity-100'}"
            title="Start Voice Recording"
          >
            <Mic class="size-5 stroke-[2.5] transition-transform group-hover/voicebtn:scale-110" />
          </button>
        {:else}
          <!-- Centered 3x Width Recording Pill (Primary theme, waveform reacting to live microphone input, only X button) -->
          <div class="h-11 px-3.5 rounded-full bg-primary text-primary-foreground shadow-xl flex items-center justify-between gap-2.5 animate-in fade-in zoom-in-95 duration-150 w-36">
            <!-- Animated Waveform Bars reacting to microphone -->
            <div class="flex items-center gap-1 h-5 flex-1 justify-center overflow-hidden">
              {#each recordingBarHeights as height, i (i)}
                <span
                  class="w-1 bg-primary-foreground/90 rounded-full transition-all duration-100 shrink-0"
                  style="height: {Math.min(height, 18)}px;"
                ></span>
              {/each}
            </div>
            
            <!-- Only X button on the right -->
            <button
              onclick={stopVoiceRecording}
              class="size-6 rounded-full hover:bg-primary-foreground/20 text-primary-foreground/90 hover:text-primary-foreground flex items-center justify-center transition-colors shrink-0 cursor-pointer"
              title="Stop Recording"
            >
              <X class="size-4" />
            </button>
          </div>
        {/if}
      </div>
      <!-- Left aligned controls -->
      <div class="flex items-center gap-3">
        <!-- Model Selection Trigger inside Prompt bottom bar -->
        <div class="relative model-picker-container">
          <button
            onclick={(e) => { e.stopPropagation(); isModelMenuOpen = !isModelMenuOpen; isToolsMenuOpen = false; isAttachMenuOpen = false; }}
            class="flex items-center gap-1.5 h-7 px-2.5 bg-muted hover:bg-muted/80 border border-border rounded-xl text-[10px] text-foreground/80 font-medium transition-all cursor-pointer"
            title="Select AI Model"
          >
            <span class="size-3.5 rounded bg-background flex items-center justify-center text-[8px] font-mono border border-border shrink-0 text-foreground mr-1">
              {#if selectedModel?.provider.includes('Google')}G{:else if selectedModel?.provider.includes('Anthropic')}A{:else if selectedModel?.provider.includes('Meta')}M{:else if selectedModel?.provider.includes('LMStudio')}LM{:else}L{/if}
            </span>
            <span>{selectedModel?.name || selectedModel?.id}{thinkingEffort !== 'off' ? ` (${thinkingEffort})` : ''}</span>
            <ChevronDown class="size-3 text-gray-500 ml-1" />
          </button>

          {#if isModelMenuOpen}
            <div
              class="absolute bottom-full left-0 mb-2 w-[320px] bg-popover border border-border text-popover-foreground rounded-xl shadow-2xl z-50 overflow-hidden flex flex-col p-0 animate-in fade-in duration-150"
            >
              <div class="p-2 border-b border-border flex items-center gap-2">
                <Search class="size-4 text-gray-500 shrink-0" />
                <input
                  type="text"
                  placeholder="Search models..."
                  class="w-full bg-transparent border-none outline-none text-xs text-foreground placeholder-muted-foreground/60 py-1"
                  bind:value={searchQuery}
                  onclick={(e) => e.stopPropagation()}
                />
              </div>
              <div class="max-h-[280px] overflow-y-auto p-1.5 flex flex-col gap-3">
                {#each groupedModels() as group}
                  <div class="flex flex-col gap-0.5">
                    <div class="text-[9px] font-semibold text-gray-500 uppercase tracking-wider px-2.5 py-1 font-mono">
                      {group.header}
                    </div>
                    {#each group.models as model}
                      <button
                        onclick={() => { selectModel(model.id); isModelMenuOpen = false; }}
                        class="w-full flex items-center justify-between px-2.5 py-2 rounded-lg text-xs text-left hover:bg-accent hover:text-accent-foreground transition-all cursor-pointer {model.id === activeSession?.selectedModelId ? 'bg-accent text-accent-foreground font-medium border border-border' : 'text-foreground/80'}"
                      >
                        <div class="flex items-center gap-2 min-w-0">
                          <span class="size-4 rounded bg-muted flex items-center justify-center text-[9px] font-mono border border-border shrink-0">
                            {#if model.provider.includes('Google')}G{:else if model.provider.includes('Anthropic')}A{:else if model.provider.includes('Meta')}M{:else if model.provider.includes('LMStudio')}LM{:else}L{/if}
                          </span>
                          <span class="truncate">{model.name}</span>
                        </div>
                        <div class="flex items-center gap-1">
                          <span
                            onclick={(e) => toggleFavorite(e, model.id)}
                            onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') toggleFavorite(e, model.id); }}
                            role="button"
                            tabindex="0"
                            class="p-1 hover:text-yellow-400 text-gray-500 transition-colors cursor-pointer"
                          >
                            <Star class="size-3 {favoriteModelIds.includes(model.id) ? 'fill-yellow-400 text-yellow-400' : ''}" />
                          </span>
                          {#if model.id === activeSession?.selectedModelId}
                            <Check class="size-3.5 text-emerald-400 shrink-0" />
                          {/if}
                        </div>
                      </button>
                    {/each}
                  </div>
                {/each}
              </div>

              <!-- Thinking Effort Slider -->
              <div class="p-2.5 border-t border-border">
                <div class="flex items-center justify-between mb-2">
                  <div class="flex items-center gap-1.5">
                    <Brain class="size-3 text-muted-foreground" />
                    <span class="text-[10px] font-semibold text-muted-foreground uppercase tracking-wider font-mono">Thinking</span>
                  </div>
                  <span class="text-[10px] font-mono text-foreground capitalize">{thinkingEffort}</span>
                </div>
                <div class="relative h-5 flex items-center" onclick={(e) => e.stopPropagation()}>
                  <!-- Track background -->
                  <div class="absolute inset-x-0 top-1/2 -translate-y-1/2 h-1.5 rounded-full bg-muted-foreground/15"></div>
                  <!-- Filled track -->
                  <div class="absolute top-1/2 -translate-y-1/2 h-1.5 rounded-full bg-primary left-0 transition-all" style="width: {thinkingIdx * 25}%"></div>
                  <!-- Stop dots and clickable areas -->
                  {#each thinkingLevels as level, i}
                    <button
                      onclick={() => { thinkingEffort = level; }}
                      class="absolute top-1/2 -translate-y-1/2 -translate-x-1/2 size-5 flex items-center justify-center cursor-pointer z-10"
                      style="left: {i * 25}%"
                      title={level}
                    >
                      <span class="size-2 rounded-full transition-all {i <= thinkingIdx ? 'bg-primary' : 'bg-muted-foreground/30'} {i === thinkingIdx ? 'size-3.5 bg-primary ring-2 ring-background shadow-sm' : ''}"></span>
                    </button>
                  {/each}
                </div>
                <div class="flex items-center justify-between mt-0.5">
                  {#each thinkingLevels as level, i}
                    <button
                      onclick={(e) => { e.stopPropagation(); thinkingEffort = level; }}
                      class="text-[9px] font-mono capitalize cursor-pointer transition-colors {i === thinkingIdx ? 'text-foreground font-medium' : 'text-muted-foreground/50 hover:text-muted-foreground'}"
                      style="width: 20%; text-align: {i === 0 ? 'left' : i === thinkingLevels.length - 1 ? 'right' : 'center'}"
                    >{level}</button>
                  {/each}
                </div>
              </div>
            </div>
          {/if}
        </div>
      </div>

      <!-- Right aligned controls -->
      <div class="flex items-center gap-1.5 ms-auto">
        <!-- Tools Extensions Popover -->
        <div class="relative tools-picker-container">
          <button
            onclick={(e) => { e.stopPropagation(); isToolsMenuOpen = !isToolsMenuOpen; isModelMenuOpen = false; isAttachMenuOpen = false; }}
            class="flex items-center gap-1.5 px-2.5 py-1 bg-muted hover:bg-muted/80 border border-border rounded-xl text-[10px] font-mono text-foreground/80 transition-all cursor-pointer h-7"
            title="Session Extensions & Tool Permissions"
          >
            <Wrench class="size-3 text-gray-400" />
            <span>{totalActiveToolsCount()}</span>
          </button>

          {#if isToolsMenuOpen}
            <div class="absolute bottom-full right-0 mb-2 w-[320px] bg-popover border border-border text-popover-foreground rounded-xl shadow-2xl z-50 overflow-hidden flex flex-col animate-in fade-in duration-150">
              <div class="p-2 border-b border-border flex items-center gap-2">
                <Search class="size-4 text-gray-500 shrink-0" />
                <input
                  type="text"
                  placeholder="Search tools..."
                  class="w-full bg-transparent border-none outline-none text-xs text-foreground placeholder-muted-foreground/60 py-1"
                  bind:value={toolsSearchQuery}
                  onclick={(e) => e.stopPropagation()}
                />
              </div>
              <div class="max-h-[280px] overflow-y-auto p-1.5 flex flex-col gap-3">
                {#each groupedExtensions() as group}
                  <div class="flex flex-col gap-0.5">
                    <div class="text-[9px] font-semibold text-gray-500 uppercase tracking-wider px-2.5 py-1 font-mono">
                      {group.header}
                    </div>
                    {#each group.items as ext (ext.id)}
                      <div
                        class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-xs text-left hover:bg-accent hover:text-accent-foreground transition-all cursor-pointer"
                        role="button"
                        tabindex="0"
                        onclick={() => handleExtensionRowClick(ext)}
                        onkeydown={(e) => { if (e.key === 'Enter') handleExtensionRowClick(ext); }}
                      >
                        <div class="flex items-center gap-2 min-w-0">
                          <span class="text-sm shrink-0">{ext.icon}</span>
                          <span class="truncate text-foreground font-medium">{ext.name}</span>
                        </div>
                        <select
                          class="bg-muted border border-border text-[10px] text-foreground px-2 py-0.5 rounded cursor-pointer outline-none"
                          value={ext.permission}
                          onclick={(e) => e.stopPropagation()}
                          onchange={(e) => setParentPermission(ext.id, (e.target as HTMLSelectElement).value as ParentPermissionState)}
                        >
                          {#if ext.tools && ext.tools.length > 0}
                            <option value="per_tool">Per Tool</option>
                          {/if}
                          <option value="off">Off</option>
                          <option value="ask">Ask</option>
                          <option value="allow">Allow</option>
                        </select>
                      </div>

                      {#if selectedExtForTools?.id === ext.id && ext.tools}
                        <div class="pl-6 flex flex-col gap-0.5 border-l border-border ml-4 mt-0.5 mb-1.5">
                          {#each ext.tools as tool (tool.id)}
                            <div class="flex items-center justify-between py-1 px-2 hover:bg-accent hover:text-accent-foreground rounded">
                              <span class="text-[11px] text-muted-foreground truncate">{tool.name}</span>
                              {#if ext.permission === 'per_tool'}
                                <select
                                  class="bg-muted border border-border text-[9px] text-foreground px-1.5 py-0.5 rounded cursor-pointer outline-none"
                                  value={tool.permission}
                                  onclick={(e) => e.stopPropagation()}
                                  onchange={(e) => setToolPermission(ext.id, tool.id, (e.target as HTMLSelectElement).value as PermissionState)}
                                >
                                  <option value="off">Off</option>
                                  <option value="ask">Ask</option>
                                  <option value="allow">Allow</option>
                                </select>
                              {/if}
                            </div>
                          {/each}
                        </div>
                      {/if}
                    {/each}
                  </div>
                {/each}
              </div>
            </div>
          {/if}
        </div>

        <!-- Attach Popover -->
        <div class="relative attach-picker-container">
          <InputGroup.Button
            variant="ghost"
            class="rounded-full size-7 flex items-center justify-center p-0 text-muted-foreground hover:text-foreground hover:bg-muted bg-transparent border-none"
            onclick={toggleAttachMenu}
            title="Attach files or folders"
          >
            <Paperclip class="size-4" />
          </InputGroup.Button>

          {#if isAttachMenuOpen}
            <div class="absolute bottom-full right-0 mb-2 w-[160px] bg-popover border border-border rounded-xl shadow-2xl z-50 p-1 flex flex-col gap-0.5 animate-in fade-in duration-100">
              <button onclick={handleAttachFiles} class="flex items-center gap-2.5 w-full px-3 py-2 text-xs rounded-lg hover:bg-accent hover:text-accent-foreground text-foreground text-left transition-all">
                <File class="size-3.5 text-muted-foreground" />
                <span>Attach Files...</span>
              </button>
              <button onclick={handleAttachFolder} class="flex items-center gap-2.5 w-full px-3 py-2 text-xs rounded-lg hover:bg-accent hover:text-accent-foreground text-foreground text-left transition-all">
                <Folder class="size-3.5 text-muted-foreground" />
                <span>Attach Folder...</span>
              </button>
            </div>
          {/if}
        </div>

        <Separator orientation="vertical" class="!h-4 bg-border" />

        <!-- Send button -->
        <InputGroup.Button
          variant="default"
          class="rounded-full size-7 flex items-center justify-center p-0"
          disabled={!promptText.trim() && attachedFiles.length === 0}
          onclick={handleSubmit}
          title="Send Message"
        >
          <ArrowUpIcon class="size-4" />
          <span class="sr-only">Send</span>
        </InputGroup.Button>
      </div>
    </InputGroup.Addon>
  </InputGroup.Root>
{/snippet}

