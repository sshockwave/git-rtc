import { assert_never, parse_message_as_json } from '../utils';
import { public_stun_servers, setup } from '../rtc/establish';
import { Peer as PeerInfo, PeerMessage, SignalEvent, SignalRequest } from '../message';
import { RTCPeerConnection } from '@roamhq/wrtc';

export type GitRtcServerInit = {
  repo_mapping: Map<string, string>;
  server_name: string;
};

type Peer = PeerInfo & {
  connection?: {
    pc: RTCPeerConnection,
    on_message: (message: PeerMessage) => void,
    channels: Map<string, RTCDataChannel>,
  };
};

export class GitRtcServer {
  repo_mapping: Map<string, string>;
  name: string;
  peer_list: Map<string, Peer> = new Map;

  constructor(options: GitRtcServerInit) {
    this.repo_mapping = new Map;
    this.name = options.server_name;
  }

  add_signal_server(url: string | URL, options?: {
    signal: AbortSignal,
  }) {
    const ws = new WebSocket(url);
    options?.signal.addEventListener('abort', () => {
      ws.close();
    });
    function send(data: SignalRequest) {
      ws.send(JSON.stringify(data));
    }
    function send_peer_list() {
      send({ action: 'fetch-peer-list' });
    }
    ws.addEventListener('open', () => {
      send({ action: 'handshake', name: this.name });
    });
    let server_peer_list = new Map<string, Peer>(); // TODO
    ws.addEventListener('message', async event => {
      try {
        const data: SignalEvent = await parse_message_as_json(event);
        if (data.action === 'full-peer-list') {
          this.peer_list = new Map(data.peers.map(peer => {
            const old_peer = this.peer_list.get(peer.id);
            if (old_peer) {
              Object.assign(old_peer, peer);
              return [peer.id, old_peer];
            }
            return [peer.id, peer];
          }));
        } else if (data.action === 'new-peer') {
          this.peer_list.set(data.peer.id, data.peer);
          if (this.peer_list.size !== data.peer_cnt) {
            send_peer_list();
          }
        } else if (data.action === 'delete-peer') {
          this.peer_list.delete(data.peer_id);
          if (this.peer_list.size !== data.peer_cnt) {
            send_peer_list();
          }
        } else if (data.action === 'receive-offer') {
          const { peer_id } = data;
          const peer = this.peer_list.get(peer_id);
          if (!peer) {
            send_peer_list();
            return;
          }
          if (!('connection' in peer)) {
            const pc = new RTCPeerConnection({
              iceServers: public_stun_servers,
            });
            peer.connection = {
              pc,
              on_message: setup(pc, message => send({
                action: 'offer',
                peer_id: peer_id,
                message,
              })),
              channels: new Map,
            };
            pc.addEventListener('connectionstatechange', () => {
              console.log('connection state:', pc.connectionState);
            });
            pc.addEventListener('signalingstatechange', () => {
              console.log('signaling state:', pc.signalingState);
            });
          }
          peer.connection?.on_message(data.message);
        } else {
          assert_never(data);
        }
      } catch (e) {
        console.error(e);
      }
    });
  }
}
