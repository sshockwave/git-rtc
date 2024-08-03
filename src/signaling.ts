import { IncomingMessage } from "node:http";
import type WebSocket from "ws";
import { randomUUID } from "node:crypto";
import { SignalEvent, SignalRequest } from "./message";
import { send } from "node:process";

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
  socket.on('message', message => {
    const data: SignalRequest = JSON.parse(message.toString('utf-8'));
    try {
      if (!started) {
        if (data.action !== 'handshake') {
          throw new Error('handshake required');
        }
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
        client.send({
          action: 'full-peer-list',
          peers: Array.from(client_list.values()).map(({ name, id }) => ({ name, id })),
        });
        client_list.set(client.id, client);
      }
    } catch {
      console.error('invalid message from client:', data);
    }
  });
  socket.on('close', () => {
    if (started) {
      client_list.delete(client.id);
      for (const { send } of client_list.values()) {
        send({
          action: 'delete-peer',
          peer_id: client.id,
          peer_cnt: client_list.size,
        });
      }
    }
  });
}
