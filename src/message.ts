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
} | {
  action: 'receive-offer',
  peer_id: string,
  message: PeerMessage,
};

export type PeerMessage = PeerMessageInit & { is_polite: boolean };
export type PeerMessageInit = {
  description: RTCSessionDescriptionInit,
} | {
  candidate: RTCIceCandidateInit,
};

export type SignalRequest = {
  action: 'handshake',
  name: string,
} | {
  action: 'offer',
  peer_id: string,
  message: PeerMessageInit,
} | {
  action: 'fetch-peer-list',
};
