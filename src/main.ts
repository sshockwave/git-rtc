import { createServer } from './server';
// create an express server
import express from 'express';
import expressWs from 'express-ws';
import { handleSocket } from './signaling';
import type { WebSocket } from 'ws';
import { createProxyMiddleware } from 'http-proxy-middleware';

const dev = process.env.NODE_ENV !== "production";

async function main() {
  const app = express();
  // expressWs(app);
  // (app as any).ws('/app', (ws: WebSocket) => {
  //   console.log('here');
  //   handleSocket(ws);
  // });
  const wsProxy = createProxyMiddleware({
    target: 'ws://localhost:9243',
    changeOrigin: true,
    ws: true,
  });
  app.use(wsProxy);
  app.use('/', createProxyMiddleware({
    target: 'http://localhost:9243',
    ws: true,
    changeOrigin: true,
  }));
  const server = app.listen(9242, () => {
    console.log(`Server listening at http://localhost:9242`);
  });
  server.on('upgrade', wsProxy.upgrade);
}

main()
