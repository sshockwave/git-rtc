'use client';

import { useAbort } from "@/utils/hook";
import styles from "./page.module.css";
import { useState } from "react";
import { Peer, SignalEvent, SignalRequest } from '../../src/message';

const red_emoji = '\u{1F534}';
const yellow_emoji = '\u{1F7E1}';
const green_emoji = '\u{1F7E2}';

function PeerDisplay({ peer }: { peer: Peer }) {
  return <div>
    {peer.name} {peer.id}
  </div>;
}

function assert_never(x: never): never {
  throw new Error('unexpected object: ' + x);
}

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
  const [peer_list, set_peer_list] = useState<Map<string, Peer>>(new Map);
  useAbort(signal => {
    const ws = new WebSocket(server_url);
    const decoder = new TextDecoder('utf-8');
    function update_state() {
      if (!signal.aborted) {
        setReadyState(ws.readyState);
      }
    }
    function send(data: SignalRequest) {
      ws.send(JSON.stringify(data));
    }
    ws.addEventListener('open', () => {
      setReadyState(ws.readyState);
      send({ action: 'handshake', name: 'browser' });
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
        const data: SignalEvent = JSON.parse(buffer);
        if (data.action === 'full-peer-list') {
          set_peer_list(new Map(data.peers.map(peer => [peer.id, peer])));
        } else if (data.action === 'new-peer') {
          const new_peer_list = new Map(peer_list);
          new_peer_list.set(data.peer.id, data.peer);
          if (new_peer_list.size !== data.peer_cnt) {
            send({ action: 'fetch-peer-list' });
          }
          set_peer_list(new_peer_list);
        } else if (data.action === 'delete-peer') {
          const new_peer_list = new Map(peer_list);
          new_peer_list.delete(data.peer_id);
          if (new_peer_list.size !== data.peer_cnt) {
            send({ action: 'fetch-peer-list' });
          }
          set_peer_list(new_peer_list);
        } else {
          assert_never(data);
        }
      } catch (e) {
        console.error(e);
      }
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
    <ul>
      {Array.from(peer_list.values()).map(peer => <li><PeerDisplay key={peer.id} peer={peer} /></li>)}
    </ul>
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
