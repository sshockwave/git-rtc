import { PeerMessage, PeerMessageInit } from "../message";
import { assert_never } from "../utils";

export const public_stun_servers: RTCIceServer[] = [
  { urls: 'stun:stun.l.google.com:19302' },
  { urls: 'stun:stun.voipbuster.com' },
  { urls: 'stun:stun.wirlab.com' },
  { urls: 'stun:stun.voipstunt.com' },
  { urls: 'stun:freeturn.net:3479' },
  // { urls: 'turn:freeturn.net:3479', username: 'free', credential: 'free' },
];

// https://developer.mozilla.org/en-US/docs/Web/API/WebRTC_API/Perfect_negotiation
export function setup(pc: RTCPeerConnection, send: (req: PeerMessageInit) => void) {
  /**
   * Are we trying to send an offer?
   * We can test this using `(making_offer || pc.signalingState !== "stable")`
   * since making_offer is true when creating the offer asynchronously,
   * and signalingState changes to "have-local-offer" after the offer is created.
   */
  let making_offer: false | Promise<void> = false;
  // TODO: support AbortSignal
  pc.addEventListener('negotiationneeded', async () => {
    let res: (value: void) => void;
    making_offer = new Promise(r => res = r);
    try {
      const description = await pc.createOffer();
      await pc.setLocalDescription(description);
      send({ description });
    } catch (err) {
      console.error(err);
    } finally {
      res!();
      making_offer = false;
    }
  });
  pc.addEventListener('icecandidate', ({ candidate }) => {
    if (candidate) {
      send({ candidate });
    }
  });
  let ignore_offer = false;
  return async (data: PeerMessage) => {
    try {
      if ('description' in data) {
        const { description } = data;
        ignore_offer = false;
        // offer collision
        if (description.type === "offer" && (making_offer || pc.signalingState !== "stable")) {
          // When the peer is polite, ignore its offer if we are already making an offer
          if (data.is_polite) {
            ignore_offer = true;
            return;
          }
          // Otherwise, rollback the local description and use the remote description
          await making_offer;
          await pc.setLocalDescription({ type: 'rollback' });
        }
        await pc.setRemoteDescription(description);
        if (description.type === 'offer') {
          const answer = await pc.createAnswer();
          await pc.setLocalDescription(answer);
          send({ description: answer });
        }
      } else if ('candidate' in data) {
        const { candidate } = data;
        try {
          await pc.addIceCandidate(candidate);
        } catch (err) {
          if (!ignore_offer) {
            throw err;
          }
        }
      }
    } catch (err) {
      console.error(err);
    }
  };
}
