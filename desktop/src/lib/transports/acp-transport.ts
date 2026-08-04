import * as acp from '@agentclientprotocol/sdk';
import type { ExtendedAcpStream } from './tauri-ipc-transport';

export function createMockAcpStream(): ExtendedAcpStream {
  let controller: ReadableStreamDefaultController<acp.AnyMessage>;

  const readable = new ReadableStream<acp.AnyMessage>({
    start(c) {
      controller = c;
    }
  });

  const writable = new WritableStream<acp.AnyMessage>({
    write(message) {
      if ('id' in message && 'method' in message) {
        const msg = message as acp.AnyRequest;
        if (msg.method === acp.methods.agent.initialize) {
          setTimeout(() => {
            controller.enqueue({
              jsonrpc: '2.0',
              id: msg.id,
              result: {
                protocolVersion: acp.PROTOCOL_VERSION,
                agentInfo: { name: 'Mock Agent', version: '0.1.0' }
              }
            });
          }, 50);
        } else if (msg.method === acp.methods.agent.session.new) {
          setTimeout(() => {
            controller.enqueue({
              jsonrpc: '2.0',
              id: msg.id,
              result: {
                sessionId: 'mock-sess-1',
                configOptions: [
                  { category: 'model', id: 'model', options: [{ id: 'mock-gpt-4o', name: 'Mock GPT-4o', group: 'Mock' }] }
                ]
              }
            });
          }, 50);
        } else if (msg.method === acp.methods.agent.session.prompt) {
          setTimeout(() => {
            controller.enqueue({
              jsonrpc: '2.0',
              method: acp.methods.client.session.update,
              params: {
                sessionId: 'mock-sess-1',
                update: {
                  sessionUpdate: 'agent_message_chunk',
                  content: { type: 'text', text: 'This is a mock response from the ACP Mock Stream.' }
                }
              }
            });
            controller.enqueue({
              jsonrpc: '2.0',
              id: msg.id,
              result: { stopReason: 'end_turn' }
            });
          }, 200);
        }
      }
    }
  });

  return { readable, writable };
}
