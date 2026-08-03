import type { AcpTransport, AcpMessageNotification } from './acp-transport';
import type { AcpModel } from '../types/acp';

export class AcpClientSession {
  private transport: AcpTransport;
  private isInitialized = false;
  public fetchedModels: AcpModel[] = [];

  constructor(transport: AcpTransport) {
    this.transport = transport;
  }

  async startSession(onNotification: (notif: AcpMessageNotification) => void): Promise<AcpModel[]> {
    await this.transport.connect();
    this.transport.onNotification(onNotification);

    const initResult = await this.transport.sendRequest('initialize', {
      clientInfo: { name: 'Hirn Desktop', version: '0.1.0' },
      protocolVersion: '1.0.0'
    });

    if (initResult && initResult.models) {
      this.fetchedModels = initResult.models;
    }

    await this.transport.sendNotification('notifications/initialized', {});
    this.isInitialized = true;
    console.log('[AcpClientSession] Handshake initialized:', initResult);

    return this.fetchedModels;
  }

  async sendPrompt(promptText: string, modelId?: string): Promise<void> {
    if (!this.isInitialized) {
      throw new Error('ACP Client Session is not initialized.');
    }
    await this.transport.sendRequest('session/prompt', { prompt: promptText, modelId });
  }
}
