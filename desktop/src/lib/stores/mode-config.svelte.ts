import type { SessionStore } from './session.svelte';
import type { AcpModeConfig, ThoughtLevelConfig } from '../types/acp';

export class ModeConfigStore {
  private sessionStore: SessionStore;

  constructor(sessionStore: SessionStore) {
    this.sessionStore = sessionStore;
  }

  get activeSession() {
    return this.sessionStore.activeSession;
  }

  get modeConfig(): AcpModeConfig | null {
    return this.sessionStore.activeSession?.modeConfig ?? null;
  }

  get currentMode() {
    const config = this.modeConfig;
    if (!config) return null;
    return config.modes.find(m => m.id === config.currentModeId) ?? config.modes[0] ?? null;
  }

  get thoughtLevelConfig(): ThoughtLevelConfig | null {
    return this.sessionStore.activeSession?.thoughtLevelConfig ?? null;
  }

  get currentThinkingLevel(): string {
    return this.thoughtLevelConfig?.currentValue ?? 'off';
  }

  get isThinkingSupported(): boolean {
    return !!this.thoughtLevelConfig;
  }

  get hasThinkingOptions(): boolean {
    return (this.thoughtLevelConfig?.levels?.length ?? 0) > 1;
  }

  async setMode(modeId: string) {
    const session = this.sessionStore.activeSession;
    if (session) {
      await this.sessionStore.setMode(session.id, modeId);
    }
  }

  async setThinkingLevel(level: string) {
    const session = this.sessionStore.activeSession;
    if (session) {
      await this.sessionStore.setThinkingLevel(session.id, level);
    }
  }
}
