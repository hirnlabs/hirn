export interface KeybindingDef {
  id: string;
  command: string;
  key: string;
  context: string;
  label: string;
  displayKey: string;
}

export type CommandHandler = (args?: any) => void;

class KeybindingStore {
  keybindings = $state<KeybindingDef[]>([
    // Global Trigger Commands
    { id: 'toggle-model', command: 'popover:toggle-model', key: 'ctrl+m', context: 'global', label: 'Model Selector', displayKey: 'Ctrl+M' },
    { id: 'toggle-mode', command: 'popover:toggle-mode', key: 'ctrl+a', context: 'global', label: 'Agent Mode', displayKey: 'Ctrl+A' },
    { id: 'toggle-tools', command: 'popover:toggle-tools', key: 'ctrl+t', context: 'global', label: 'Tools Extensions', displayKey: 'Ctrl+T' },
    { id: 'toggle-attach', command: 'popover:toggle-attach', key: 'ctrl+f', context: 'global', label: 'Attach Files', displayKey: 'Ctrl+F' },
    { id: 'toggle-recording', command: 'voice:toggle-recording', key: 'ctrl+r', context: 'global', label: 'Toggle Voice Recording', displayKey: 'Ctrl+R' },
    { id: 'toggle-settings', command: 'sidebar:toggle-settings', key: 'ctrl+c', context: 'global', label: 'Toggle Settings Sidebar', displayKey: 'Ctrl+C' },
    { id: 'close-all', command: 'popover:close-all', key: 'escape', context: 'global', label: 'Close Menus', displayKey: 'Esc' },
    { id: 'send-prompt', command: 'chat:send-prompt', key: 'enter', context: 'global', label: 'Send Message', displayKey: 'Enter' },

    // Model Popover Commands
    { id: 'model-select-1', command: 'model:select-index-0', key: '1', context: 'model_popover', label: 'Select Model 1', displayKey: '1' },
    { id: 'model-select-2', command: 'model:select-index-1', key: '2', context: 'model_popover', label: 'Select Model 2', displayKey: '2' },
    { id: 'model-select-3', command: 'model:select-index-2', key: '3', context: 'model_popover', label: 'Select Model 3', displayKey: '3' },
    { id: 'model-select-4', command: 'model:select-index-3', key: '4', context: 'model_popover', label: 'Select Model 4', displayKey: '4' },
    { id: 'model-select-5', command: 'model:select-index-4', key: '5', context: 'model_popover', label: 'Select Model 5', displayKey: '5' },
    { id: 'model-configure', command: 'sidebar:open-settings', key: 'c', context: 'model_popover', label: 'More Options', displayKey: 'C' },

    // Mode Popover Commands
    { id: 'mode-select-1', command: 'mode:select-index-0', key: '1', context: 'mode_popover', label: 'Select Mode 1', displayKey: '1' },
    { id: 'mode-select-2', command: 'mode:select-index-1', key: '2', context: 'mode_popover', label: 'Select Mode 2', displayKey: '2' },
    { id: 'mode-select-3', command: 'mode:select-index-2', key: '3', context: 'mode_popover', label: 'Select Mode 3', displayKey: '3' },
    { id: 'thinking-off', command: 'thinking:set-off', key: 'o', context: 'mode_popover', label: 'Thinking Off', displayKey: 'O' },
    { id: 'thinking-low', command: 'thinking:set-low', key: 'l', context: 'mode_popover', label: 'Thinking Low', displayKey: 'L' },
    { id: 'thinking-mid', command: 'thinking:set-mid', key: 'm', context: 'mode_popover', label: 'Thinking Mid', displayKey: 'M' },
    { id: 'thinking-high', command: 'thinking:set-high', key: 'h', context: 'mode_popover', label: 'Thinking High', displayKey: 'H' },
    { id: 'thinking-max', command: 'thinking:set-max', key: 'x', context: 'mode_popover', label: 'Thinking Max', displayKey: 'X' },
    { id: 'mode-configure', command: 'sidebar:open-settings', key: 'c', context: 'mode_popover', label: 'More Options', displayKey: 'C' },

    // Attach Popover Commands
    { id: 'attach-files', command: 'attach:files', key: '1', context: 'attach_popover', label: 'Attach Files', displayKey: '1' },
    { id: 'attach-folder', command: 'attach:folder', key: '2', context: 'attach_popover', label: 'Attach Folder', displayKey: '2' },
  ]);

  isCtrlPressed = $state(false);

  private handlers = new Map<string, CommandHandler>();

  registerCommand(command: string, handler: CommandHandler) {
    this.handlers.set(command, handler);
  }

  unregisterCommand(command: string) {
    this.handlers.delete(command);
  }

  dispatch(command: string, args?: any): boolean {
    const handler = this.handlers.get(command);
    if (handler) {
      handler(args);
      return true;
    }
    return false;
  }

  getKeyForCommand(command: string): string | undefined {
    return this.keybindings.find(kb => kb.command === command)?.displayKey;
  }

  async loadKeybindingsYaml() {
    let yamlStr: string | null = null;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      yamlStr = await invoke<string>('load_named_config', { name: 'keybinds.yaml' });
    } catch {
      if (typeof localStorage !== 'undefined') {
        yamlStr = localStorage.getItem('hirn_keybinds_config');
      }
    }

    if (yamlStr) {
      try {
        const yaml = await import('js-yaml');
        const parsed: any = yaml.load(yamlStr);
        if (Array.isArray(parsed)) {
          this.keybindings = parsed;
        } else if (parsed && typeof parsed === 'object' && Array.isArray(parsed.keybindings)) {
          this.keybindings = parsed.keybindings;
        }
      } catch (err) {
        console.warn('Failed to parse keybinds.yaml:', err);
      }
    }
  }

  async saveKeybindingsYaml() {
    try {
      const yaml = await import('js-yaml');
      const yamlStr = yaml.dump({ keybindings: this.keybindings }, { indent: 2 });
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('save_named_config', { name: 'keybinds.yaml', content: yamlStr });
      } catch {
        if (typeof localStorage !== 'undefined') {
          localStorage.setItem('hirn_keybinds_config', yamlStr);
        }
      }
    } catch (err) {
      console.warn('Failed to save keybinds.yaml:', err);
    }
  }

  updateBinding(id: string, newKey: string) {
    const target = this.keybindings.find(kb => kb.id === id);
    if (target) {
      target.key = newKey.toLowerCase();
      target.displayKey = newKey.toUpperCase();
      this.saveKeybindingsYaml();
    }
  }

  setCtrlPressed(pressed: boolean) {
    this.isCtrlPressed = pressed;
  }

  handleKeyEvent(event: KeyboardEvent, currentContext: string): boolean {
    if (event.key === 'Control' || event.key === 'Alt' || event.key === 'Meta') {
      this.isCtrlPressed = true;
    }

    const isCtrl = event.ctrlKey || event.metaKey;
    const isAlt = event.altKey;
    const key = event.key.toLowerCase();

    // Exceptions that don't require modifier keys (navigation and escape)
    const isSpecialNavKey = key === 'escape' || key === 'enter';

    // Only allow shortcuts if Ctrl or Alt modifier is pressed (or for Esc/Enter navigation)
    if (!isCtrl && !isAlt && !isSpecialNavKey) {
      return false;
    }

    let chord = '';
    if (isCtrl) chord += 'ctrl+';
    if (isAlt) chord += 'alt+';
    chord += key;

    const matched = this.keybindings.find(kb =>
      (kb.context === currentContext || kb.context === 'global') &&
      (kb.key === chord || (isSpecialNavKey && kb.key === key))
    );

    if (matched) {
      const dispatched = this.dispatch(matched.command);
      if (dispatched) {
        event.preventDefault();
        return true;
      }
    }
    return false;
  }

  handleKeyUp(event: KeyboardEvent) {
    if (event.key === 'Control' || event.key === 'Alt' || event.key === 'Meta') {
      this.isCtrlPressed = false;
    }
  }
}

export const keybindingStore = new KeybindingStore();
keybindingStore.loadKeybindingsYaml();

