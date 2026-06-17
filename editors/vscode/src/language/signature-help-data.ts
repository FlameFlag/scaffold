import { isNonNegativeInteger } from "../wasm/number";
import { isObject } from "../wasm/object";
import { isNonEmptyString } from "../wasm/string";

export interface FormContext {
  head: string;
  active_argument: number;
}

export interface WasmSignatureHelp {
  label: string;
  documentation: string;
  parameters: WasmSignatureParameter[];
}

export interface WasmSignatureParameter {
  label: string;
  documentation?: string | null;
}

export function isFormContext(value: unknown): value is FormContext {
  return (
    isObject(value) &&
    isNonEmptyString(value.head) &&
    isNonNegativeInteger(value.active_argument)
  );
}

export function isWasmSignatureHelp(
  value: unknown,
): value is WasmSignatureHelp {
  return (
    isObject(value) &&
    isNonEmptyString(value.label) &&
    typeof value.documentation === "string" &&
    Array.isArray(value.parameters) &&
    value.parameters.every(isWasmSignatureParameter)
  );
}

function isWasmSignatureParameter(
  value: unknown,
): value is WasmSignatureParameter {
  return (
    isObject(value) &&
    isNonEmptyString(value.label) &&
    isOptionalNullableString(value.documentation)
  );
}

function isOptionalNullableString(value: unknown): boolean {
  return value === undefined || value === null || typeof value === "string";
}
