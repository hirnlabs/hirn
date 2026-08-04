import * as acp from '@agentclientprotocol/sdk';

export interface AcpStreamOptions {
  onError?: (err: any) => void;
}

export interface ExtendedAcpStream extends acp.Stream {
  pid?: number | null;
  disconnect?: () => Promise<void>;
}

export async function createTauriIpcStream(
  commandOrUrl: string,
  options?: AcpStreamOptions
): Promise<ExtendedAcpStream> {
  const parts = commandOrUrl.split(' ');
  const command = parts[0];
  const args = parts.slice(1);

  const { invoke } = await import('@tauri-apps/api/core');
  const { listen } = await import('@tauri-apps/api/event');

  const pid = await invoke<number>('spawn_acp_agent', { command, args });
  console.log(`[TauriIpcStream] Spawned ACP process PID ${pid} (${commandOrUrl})`);

  let stdoutUnlisten: (() => void) | undefined;
  let stderrUnlisten: (() => void) | undefined;

  const readable = new ReadableStream<acp.AnyMessage>({
    async start(controller) {
      stdoutUnlisten = await listen<[number, string]>('acp-stdout', (event) => {
        const [msgPid, line] = event.payload;
        if (msgPid === pid && line.trim()) {
          try {
            const parsed = JSON.parse(line);
            controller.enqueue(parsed);
          } catch {
            console.warn(`[TauriIpcStream PID ${pid}] Line parse warning:`, line);
          }
        }
      });

      stderrUnlisten = await listen<[number, string]>('acp-stderr', (event) => {
        const [msgPid, line] = event.payload;
        if (msgPid === pid && line.trim()) {
          console.warn(`[TauriIpcStream STDERR ${pid}]`, line);
          if (options?.onError) {
            options.onError({
              source: 'stdio',
              message: line.trim(),
              command: commandOrUrl,
              pid
            });
          }
        }
      });
    },
    cancel() {
      if (stdoutUnlisten) stdoutUnlisten();
      if (stderrUnlisten) stderrUnlisten();
    }
  });

  const writable = new WritableStream<acp.AnyMessage>({
    async write(message) {
      const payload = JSON.stringify(message);
      try {
        await invoke('send_acp_stdin', { pid, message: payload });
      } catch (err) {
        if (options?.onError) {
          options.onError({
            source: 'ipc',
            message: `Failed to write stdin to PID ${pid}: ${err}`,
            command: commandOrUrl,
            pid
          });
        }
        throw err;
      }
    }
  });

  return {
    readable,
    writable,
    pid,
    async disconnect() {
      if (stdoutUnlisten) stdoutUnlisten();
      if (stderrUnlisten) stderrUnlisten();
    }
  };
}
