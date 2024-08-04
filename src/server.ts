import { WebSocketServer } from "ws";
import { handleSocket } from "./signaling";
import { createProxyServer } from 'http-proxy';
import { createServer as createHttpServer } from 'node:http';

export function createDevServer() {
  const proxy = createProxyServer({
    target: 'http://localhost:9243',
    ws: true,
  });
  const wss = new WebSocketServer({ noServer: true });
  const server = createHttpServer((req, res) => {
    proxy.web(req, res);
  });
  server.on('upgrade', (req, socket, head) => {
    if (req.url === '/ws') {
      wss.handleUpgrade(req, socket, head, handleSocket);
    } else {
      proxy.ws(req, socket, head);
    }
  });
  return server;
}

export function createProdServer() {
  // TODO
  return createHttpServer((req, res) => {
  });
}

export const createServer = process.env.NODE_ENV !== "production" ? createDevServer : createProdServer;
