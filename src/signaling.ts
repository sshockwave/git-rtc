import type WebSocket from "ws";

export function handleSocket(socket: WebSocket) {
  socket.on('message', message => {
    const data = JSON.parse(message.toString('utf-8'));
    console.log('message from client:', data);
    socket.send(JSON.stringify('hi from server'));
  });
}
