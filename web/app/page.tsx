'use client';

import { useAbort, useRerender } from "@/utils/hook";
import styles from "./page.module.css";
import { useRef, useState } from "react";
import { Peer as PeerInfo, PeerMessage, PeerMessageInit, SignalEvent, SignalRequest } from '../../src/message';
import { public_stun_servers, setup } from '../../src/rtc/establish';
import { parse_message_as_json } from "../../src/utils";

const red_emoji = '\u{1F534}';
const yellow_emoji = '\u{1F7E1}';
const green_emoji = '\u{1F7E2}';

class PeerChannel extends EventTarget {
  name: string;
  id: string;
  send: (data: PeerMessageInit) => void;
  constructor(peer: PeerInfo, send: (data: SignalRequest) => void) {
    super();
    this.name = peer.name;
    this.id = peer.id;
    this.send = (data) => {
      send({ action: 'offer', peer_id: peer.id, message: data });
    };
  }
  emit_message_event(data: PeerMessage) {
    this.dispatchEvent(new CustomEvent("message", { detail: data }));
  }
};

function PeerDisplay({ peer }: { peer: PeerChannel }) {
  useAbort(signal => {
    const pc = new RTCPeerConnection({
      iceServers: public_stun_servers,
    });
    const handler = setup(pc, peer.send);
    peer.addEventListener('message', (event) => {
      handler((event as CustomEvent).detail);
    }, { signal });
    signal.addEventListener('abort', () => {
      pc.close();
    });
    window.addEventListener('beforeunload', () => {
      pc.close();
    }, { signal });
    const channel = pc.createDataChannel('git-rtc-control');
    pc.addEventListener('connectionstatechange', () => {
      console.log('connection state:', pc.connectionState);
    });
  }, [peer]);
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
  const [get_peer_list, set_get_peer_list] = useState<() => Map<string, PeerChannel>>(() => () => new Map);
  const rerender = useRerender();
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
    let peer_list = new Map<string, PeerChannel>();
    set_get_peer_list(() => () => peer_list);
    ws.addEventListener('message', async (event) => {
      try {
        const data: SignalEvent = await parse_message_as_json(event);
        if (data.action === 'full-peer-list') {
          peer_list = new Map(data.peers.map(peer => [peer.id, new PeerChannel(peer, send)]));
          rerender();
        } else if (data.action === 'new-peer') {
          peer_list.set(data.peer.id, new PeerChannel(data.peer, send));
          if (peer_list.size !== data.peer_cnt) {
            send({ action: 'fetch-peer-list' });
          }
          rerender();
        } else if (data.action === 'delete-peer') {
          peer_list.delete(data.peer_id);
          if (peer_list.size !== data.peer_cnt) {
            send({ action: 'fetch-peer-list' });
          }
          rerender();
        } else if (data.action === 'receive-offer') {
          const peer = peer_list.get(data.peer_id);
          if (peer) {
            peer.emit_message_event(data.message);
          }
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
    window.addEventListener('beforeunload', () => {
      ws.close();
    }, { signal });
  }, [server_url]);
  return <div>
    {websocket_state_emoji[readyState]}
    {typeof server_url === 'string' ? server_url : server_url.href}
    <ul>
      {Array.from(get_peer_list().values()).map(peer => <li key={peer.id}><PeerDisplay peer={peer} /></li>)}
    </ul>
  </div>;
}

export default function Home() {
  const [url, setURL] = useState<URL | null>(null);
  useAbort(signal => {
    const url = new URL(window.location.href);
    url.protocol = url.protocol === 'https:' ? 'wss:' : 'ws:';
    url.pathname = '/git-rtc-ws';
    setURL(url);
  }, []);
  return (
    <main>
      {url !== null ? <SignalingServer server_url={url} /> : null}
    </main>
  );
}
