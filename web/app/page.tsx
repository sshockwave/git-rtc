'use client';

import { useAbort } from "@/utils/hook";
import styles from "./page.module.css";
import { useState } from "react";
import { SignalingData } from '../../src/message';

const red_emoji = '\u{1F534}';
const yellow_emoji = '\u{1F7E1}';
const green_emoji = '\u{1F7E2}';

const websocket_state_emoji: Record<number, string> = {
  [WebSocket.CONNECTING]: yellow_emoji,
  [WebSocket.OPEN]: green_emoji,
  [WebSocket.CLOSING]: red_emoji,
  [WebSocket.CLOSED]: red_emoji,
}
function SignalingServer({ server_url }: {
  server_url: string | URL;
}) {
  const [readyState, setReadyState] = useState<number>(0);
  useAbort(signal => {
    const ws = new WebSocket(server_url);
    const decoder = new TextDecoder('utf-8');
    function update_state() {
      if (!signal.aborted) {
        setReadyState(ws.readyState);
      }
    }
    ws.addEventListener('open', () => {
      update_state();
      ws.send('hello from client');
    }, { signal });
    ws.addEventListener('message', async (event) => {
      try {
        let buffer: ArrayBuffer;
        switch (ws.binaryType) {
          case 'blob':
            buffer = await event.data.arrayBuffer();
            break;
          case 'arraybuffer':
            buffer = event.data;
            break;
          default:
            throw new Error('unknown binaryType: ', ws.binaryType);
        }
        const data: SignalingData = JSON.parse(decoder.decode(buffer));
      } catch (e) {
        console.error(e);
      }
      // TODO
      update_state();
    }, { signal });
    ws.addEventListener('error', () => {
      update_state();
    });
    ws.addEventListener('close', () => {
      update_state();
    });
    signal.addEventListener('abort', () => {
      ws.close();
    });
  }, [server_url]);
  return <div>
    {websocket_state_emoji[readyState]}
    {typeof server_url === 'string' ? server_url : server_url.href}
  </div>;
}

export default function Home() {
  return (
    <main>
      <SignalingServer server_url={'ws://localhost:9243/ws'} />
    </main>
  );
}
