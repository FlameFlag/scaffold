export function assert(condition: unknown, label: string): asserts condition {
  if (!condition) {
    throw new Error(`Missing ${label}`);
  }
}
