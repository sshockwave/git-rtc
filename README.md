# git-rtc

## Why JavaScript?

I believe that js is one of the more permanent languages with good portability, open web standards without depending on a single company or compiler, and probably the widest end-user support. Rust is a more efficient alternative which can be compiled to WebAssembly, but I consider it a transitional technology before a mathematically provable language arrives.

## Project Structure

Runtime requirements:

- WebSocket ([WHATWG](https://websockets.spec.whatwg.org/#the-websocket-interface), [MDN](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket), Node.js 22)
- WebRTC ([W3C](https://www.w3.org/TR/webrtc/), [MDN](https://developer.mozilla.org/en-US/docs/Web/API/WebRTC_API))
