export type Peer = {
  name: string,
  id: string,
};

export type SignalEvent = {
  action: 'new-peer',
  peer: Peer,
  peer_cnt: number,
} | {
  action: 'delete-peer',
  peer_id: string,
  peer_cnt: number,
} | {
  action: 'full-peer-list',
  peers: Peer[],
};

export type SignalRequest = {
  action: 'handshake',
  name: string,
} | {
  action: 'offer-sdp',
  peer_id: string,
} | {
  action: 'fetch-peer-list',
};
