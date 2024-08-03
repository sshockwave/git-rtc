import { AddressInfo } from 'node:net';
import { createServer } from './server';
import { WebSocket } from 'ws';
import { SignalRequest } from './message';
import { hostname } from 'node:os';

async function main() {
  const server = createServer();
  const address = await new Promise<string>((res, rej) => {
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
  while (true) {
    try {
      const ws = new WebSocket(ws_addr);
      function send(data: SignalRequest) {
        ws.send(JSON.stringify(data));
      }
      ws.on('open', () => {
        send({ action: 'handshake', name: hostname() });
      });
      ws.on('message', message => {
        try {
          const data: SignalRequest = JSON.parse(message.toString('utf-8'));
          console.log(data);
        } catch (e) {
        }
      });
      break;
    } catch (e) {
      console.error(e);
      await new Promise(res => setTimeout(res, 1000));
    }
  }
}

main()
