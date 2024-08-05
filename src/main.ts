import { AddressInfo } from 'node:net';
import { createServer } from './server';
import { GitRtcServer } from './rtc/node-server';
import { generate_default_server_options } from './rtc/node-defaults';

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
  const git_rtc_server = new GitRtcServer(await generate_default_server_options());
  const ws_addr = new URL(address);
  ws_addr.protocol = ws_addr.protocol === 'http:' ? 'ws:' : 'wss:';
  ws_addr.pathname = '/git-rtc-ws';
  git_rtc_server.add_signal_server(ws_addr);
}

main()
