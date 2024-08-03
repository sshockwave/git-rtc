import Fastify from "fastify";
import fastifyWebSocket from '@fastify/websocket';
import fastifyHttpProxy from "@fastify/http-proxy";

export function createServer() {
  const fastify = Fastify();
  fastify.register(fastifyWebSocket);
  fastify.register(async function (fastify) {
    fastify.get('/ws', { websocket: true }, (socket, req) => {
      socket.on('message', message => {
        // message.toString() === 'hi from client'
        socket.send('hi from server');
      });
    });
  });

  fastify.register(fastifyHttpProxy, {
    upstream: 'http://127.0.0.1:9243',
  });

  return fastify;
}
