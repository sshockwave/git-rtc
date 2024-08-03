import { WebSocketServer } from "ws";
import { handleSocket } from "./signaling";
import httpProxy from 'http-proxy';
import { createServer as createHttpServer } from 'node:http';

export function createServer() {
  const proxy = httpProxy.createProxyServer({
    target: 'http://localhost:9243',
    ws: true,
  });
  const wss = new WebSocketServer({ noServer: true });
  const server = createHttpServer((req, res) => {
    proxy.web(req, res);
  });
  server.on('upgrade', (req, socket, head) => {
    if (req.url === '/ws') {
      console.log('ws connection');
      wss.handleUpgrade(req, socket, head, (ws, req) => {
        handleSocket(ws);
      });
    } else {
      proxy.ws(req, socket, head);
    }
  });
  return server;
}
