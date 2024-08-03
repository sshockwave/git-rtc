import Fastify from "fastify";
import fastifyWebSocket from '@fastify/websocket';

export function createServer() {
  const fastify = Fastify();
  fastify.register(fastifyWebSocket);
  fastify.get('/ws', { websocket: true }, (socket, req) => {
    console.log('here!');
  });

  return fastify;
}
