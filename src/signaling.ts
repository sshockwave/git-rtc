import type WebSocket from "ws";

export function handleSocket(socket: WebSocket) {
  socket.on('message', message => {
    console.log('message from client:', message);
    socket.send('hi from server');
  });
}
