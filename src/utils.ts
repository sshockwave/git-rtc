export function assert_never(value: never): never {
  throw new Error(`Unhandled value: ${value}`);
}

export async function parse_message_as_json(event: MessageEvent) {
  let buffer = event.data;
  if (buffer instanceof Blob) {
    buffer = await event.data.arrayBuffer();
  }
  if (buffer instanceof ArrayBuffer) {
    buffer = (new TextDecoder).decode(buffer);
  }
  if (typeof buffer !== 'string') {
    throw new Error(`Unexpected message type: ${typeof buffer}`);
  }
  return JSON.parse(buffer);
}

class AssertionError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'AssertionError';
  }
}

export function assert(condition: unknown, message?: string): asserts condition {
  if (!condition) {
    throw new AssertionError(message ?? '');
  }
}
