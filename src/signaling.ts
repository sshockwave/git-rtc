import { IncomingMessage } from "node:http";
import WebSocket from "ws";
import { randomUUID } from "node:crypto";
import { SignalEvent, SignalRequest } from "./message";
import { assert_never } from "./utils";

type ClientInfo = {
  address: string[],
  id: string,
  name: string,
  send: (data: SignalEvent) => void,
};

const client_list = new Map<string, ClientInfo>();

export function handleSocket(socket: WebSocket, request: IncomingMessage) {
  const client: ClientInfo = {
    address: request.socket.remoteAddress === undefined ? [] : [request.socket.remoteAddress],
    id: randomUUID(),
    name: '',
    send: (data: SignalEvent) => socket.send(JSON.stringify(data)),
  };
  const trust_forward_headers = true;
  if (trust_forward_headers) {
    const address = request.headers['x-forwarded-for'];
    if (typeof address === 'string') {
      client.address.push(address);
    } else if (address instanceof Array) {
      client.address = address.concat(client.address);
    }
  }
  let started = false;
  function send_peer_list() {
    client.send({
      action: 'full-peer-list',
      peers: Array.from(client_list.values()).map(({ name, id }) => ({ name, id })),
    });
  }
  socket.on('message', message => {
    const data: SignalRequest = JSON.parse(message.toString('utf-8'));
    try {
      if (!started && data.action !== 'handshake') {
        throw new Error('handshake required');
      }
      if (data.action === 'handshake') {
        started = true;
        client.name = data.name;
        const peer_info = {
          name: client.name,
          id: client.id,
        };
        for (const { send } of client_list.values()) {
          send({
            action: 'new-peer',
            peer: peer_info,
            peer_cnt: client_list.size,
          });
        }
        send_peer_list();
        client_list.set(client.id, client);
        return;
      } else if (data.action === 'fetch-peer-list') {
        send_peer_list();
      } else if (data.action === 'offer') {
        const peer = client_list.get(data.peer_id);
        if (peer) {
          const is_polite = data.peer_id.localeCompare(client.id) > 0;
          peer.send({
            action: 'receive-offer',
            peer_id: client.id,
            message: {
              ...data.message,
              is_polite,
            },
          });
        }
      } else {
        assert_never(data);
      }
    } catch {
      console.error('invalid message from client:', data);
    }
  });
  socket.on('error', err => {
    console.log('socket error:', err);
    if (socket.readyState === WebSocket.CLOSING || socket.readyState === WebSocket.CLOSED) {
      close_socket_cleanup();
    }
  });
  function close_socket_cleanup() {
    if (client_list.delete(client.id)) {
      for (const { send } of client_list.values()) {
        send({
          action: 'delete-peer',
          peer_id: client.id,
          peer_cnt: client_list.size,
        });
      }
    }
  }
  socket.on('close', () => {
    close_socket_cleanup();
  });
}
