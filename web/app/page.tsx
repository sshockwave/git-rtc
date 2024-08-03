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
      setReadyState(ws.readyState);
      ws.send(JSON.stringify('hello from client'));
    }, { signal });
    ws.addEventListener('message', async (event) => {
      try {
        let buffer = event.data;
        if (buffer instanceof Blob) {
          buffer = await event.data.arrayBuffer();
        }
        if (buffer instanceof ArrayBuffer) {
          buffer = decoder.decode(buffer);
        }
        if (typeof buffer !== 'string') {
          throw new Error('unknown binaryType: ' + ws.binaryType);
        }
        const data: SignalingData = JSON.parse(buffer);
        console.log(data);
      } catch (e) {
        console.error(e);
      }
      // TODO
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
  const [url, setURL] = useState<URL | null>(null);
  useAbort(signal => {
    const url = new URL(window.location.href);
    url.protocol = url.protocol === 'https:' ? 'wss:' : 'ws:';
    url.pathname = '/ws';
    setURL(url);
  }, []);
  return (
    <main>
      {url !== null ? <SignalingServer server_url={url} /> : null}
    </main>
  );
}
