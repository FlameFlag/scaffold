import { isPositiveInteger } from "./number";
import { isWasmPosition, type WasmPositionLike } from "./position";

export type WasmRangeLike = {
  line: number;
  start: number;
  length: number;
} & WasmPositionLike;

export function isWasmRange(value: unknown): value is WasmRangeLike {
  return (
    isWasmPosition(value) &&
    isPositiveInteger((value as { length?: unknown }).length)
  );
}
