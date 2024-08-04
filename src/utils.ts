export function assert_never(value: never): never {
  throw new Error(`Unhandled value: ${value}`);
}
