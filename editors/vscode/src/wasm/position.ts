import { isNonNegativeInteger } from "./number";
import { isObject } from "./object";

export type WasmPositionLike = {
  line: number;
  start: number;
};

export function isWasmPosition(value: unknown): value is WasmPositionLike {
  return (
    isObject(value) &&
    isNonNegativeInteger(value.line) &&
    isNonNegativeInteger(value.start)
  );
}
