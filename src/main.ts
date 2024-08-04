import { AddressInfo } from 'node:net';
import { createServer } from './server';
import { WebSocket } from 'ws';
import { PeerMessage, SignalEvent, SignalRequest } from './message';
import { hostname } from 'node:os';
import { assert_never } from './utils';
import { Peer as PeerInfo } from './message';
import { setup } from './rtc/establish';
import { RTCPeerConnection } from '@roamhq/wrtc';
import assert from 'node:assert';

type Peer = PeerInfo & ({
  pc: RTCPeerConnection,
  on_message: (message: PeerMessage) => void,
} | {});

async function main() {
  const server = createServer();
  const address = await new Promise<string>(res => {
    server.listen(9242, '127.0.0.1', () => {
      let address = server.address();
      if (typeof address === 'object') {
        address = `http://${(address as AddressInfo).address}:${(address as AddressInfo).port}`;
      }
      console.log(`Server listening at ${address}`);
      res(address);
    });
  });
  const ws_addr = new URL(address);
  ws_addr.protocol = ws_addr.protocol === 'http:' ? 'ws:' : 'wss:';
  ws_addr.pathname = '/ws';
      const ws = new WebSocket(ws_addr);
      function send(data: SignalRequest) {
        ws.send(JSON.stringify(data));
      }
      function send_peer_list() {
        send({ action: 'fetch-peer-list' });
      }
      ws.on('open', () => {
        send({ action: 'handshake', name: hostname() });
      });
      let peer_list: Map<string, Peer> = new Map;
      ws.on('message', message => {
        try {
          const data: SignalEvent = JSON.parse(message.toString('utf-8'));
          if (data.action === 'full-peer-list') {
            peer_list = new Map(data.peers.map(peer => {
              const old_peer = peer_list.get(peer.id);
              if (old_peer) {
                Object.assign(old_peer, peer);
                return [peer.id, old_peer];
              }
              return [peer.id, peer];
            }));
          } else if (data.action === 'new-peer') {
            peer_list.set(data.peer.id, data.peer);
            if (peer_list.size !== data.peer_cnt) {
              send_peer_list();
            }
          } else if (data.action === 'delete-peer') {
            peer_list.delete(data.peer_id);
            if (peer_list.size !== data.peer_cnt) {
              send_peer_list();
            }
          } else if (data.action === 'receive-offer') {
            const { peer_id } = data;
            const peer = peer_list.get(peer_id);
            if (!peer) {
              send_peer_list();
              return;
            }
            if (!('pc' in peer)) {
              const pc = new RTCPeerConnection();
              Object.assign(peer, {
                pc,
                on_message: setup(pc, message => send({
                  action: 'offer',
                  peer_id: peer_id,
                  message,
                })),
              });
              pc.addEventListener('connectionstatechange', () => {
                console.log('connection state:', pc.connectionState);
              });
              pc.addEventListener('signalingstatechange', () => {
                console.log('signaling state:', pc.signalingState);
              });
              pc.addEventListener('datachannel', e => {
                console.log('new datachannel:', e);
              });
            }
            assert('pc' in peer);
            peer.on_message(data.message);
          } else {
            assert_never(data);
          }
        } catch (e) {
          console.error(e);
        }
      });
}

main()
