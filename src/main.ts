import { createServer } from './server';
import { AddressInfo } from 'node:net';
// create an express server
import { createProxyMiddleware } from 'http-proxy-middleware';

const dev = process.env.NODE_ENV !== "production";

async function main() {
  const server = createServer();
  server.listen(9242, '127.0.0.1', () => {
    let address = server.address();
    if (typeof address === 'object') {
      address = `http://${(address as AddressInfo).address}:${(address as AddressInfo).port}`;
    }
    console.log(`Server listening at ${address}`);
  });
}

main()
