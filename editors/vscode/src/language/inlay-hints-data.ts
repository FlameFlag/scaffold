import { isObject } from "../wasm/object";
import { isWasmPosition } from "../wasm/position";
import { isNonEmptyString } from "../wasm/string";

export interface WasmInlayHint {
  line: number;
  start: number;
  label: string;
  tooltip: string;
}

export function isWasmInlayHint(value: unknown): value is WasmInlayHint {
  if (!isObject(value) || !isWasmPosition(value)) {
    return false;
  }
  const hint = value as Record<string, unknown>;
  return isNonEmptyString(hint.label) && typeof hint.tooltip === "string";
}
