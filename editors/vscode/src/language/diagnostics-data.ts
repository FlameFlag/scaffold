import { isNonNegativeInteger } from "../wasm/number";
import { isObject } from "../wasm/object";

export const missingDocCode = "scaffold::dsl::missing-doc";

export interface WasmDiagnostic {
  message: string;
  offset: number;
  length: number;
  severity: WasmDiagnosticSeverity;
  code: string;
  data?: WasmDiagnosticData;
}

export type WasmDiagnosticSeverity =
  | "error"
  | "warning"
  | "information"
  | "hint";

export type WasmDiagnosticData = {
  name?: string;
  line?: number;
};

type WasmDiagnosticValidator = (value: Record<string, unknown>) => boolean;

const wasmDiagnosticValidators: WasmDiagnosticValidator[] = [
  hasDiagnosticRange,
  hasDiagnosticText,
  hasDiagnosticSeverity,
  hasOptionalDiagnosticData,
];

export function diagnosticLineNumber(
  diagnosticData: WasmDiagnosticData | undefined,
  fallbackLine: number,
  lineCount: number,
): number | undefined {
  const line = diagnosticData?.line ?? fallbackLine;
  return line >= 0 && line < lineCount ? line : undefined;
}

export function isWasmDiagnostic(value: unknown): value is WasmDiagnostic {
  return (
    isObject(value) && wasmDiagnosticValidators.every((valid) => valid(value))
  );
}

function hasDiagnosticRange(value: Record<string, unknown>): boolean {
  return (
    isNonNegativeInteger(value.offset) && isNonNegativeInteger(value.length)
  );
}

function hasDiagnosticText(value: Record<string, unknown>): boolean {
  return typeof value.message === "string" && typeof value.code === "string";
}

function hasDiagnosticSeverity(value: Record<string, unknown>): boolean {
  return isDiagnosticSeverity(value.severity);
}

function hasOptionalDiagnosticData(value: Record<string, unknown>): boolean {
  return isOptionalDiagnosticData(value.data);
}

function isDiagnosticSeverity(value: unknown): value is WasmDiagnosticSeverity {
  return (
    value === "error" ||
    value === "warning" ||
    value === "information" ||
    value === "hint"
  );
}

function isOptionalDiagnosticData(
  value: unknown,
): value is WasmDiagnosticData | undefined {
  return value === undefined || isDiagnosticData(value);
}

function isDiagnosticData(value: unknown): value is WasmDiagnosticData {
  return (
    isObject(value) &&
    isOptionalString(value.name) &&
    isOptionalNonNegativeInteger(value.line)
  );
}

function isOptionalString(value: unknown): boolean {
  return value === undefined || typeof value === "string";
}

function isOptionalNonNegativeInteger(value: unknown): boolean {
  return value === undefined || isNonNegativeInteger(value);
}
