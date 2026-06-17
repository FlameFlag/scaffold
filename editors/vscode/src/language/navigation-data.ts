import { isObject } from "../wasm/object";
import { isWasmRange, type WasmRangeLike } from "../wasm/range";
import { isNonEmptyString } from "../wasm/string";

export interface WasmDocumentSymbol extends WasmRangeLike {
  name: string;
  detail?: string | null;
  kind: WasmSymbolKind;
}

export interface WasmDefinitionLocation extends WasmRangeLike {
  uri: string;
}

export interface WasmWorkspaceSymbol extends WasmDefinitionLocation {
  name: string;
  kind: WasmSymbolKind;
  group?: string;
  deprecated: boolean;
}

export type WasmSymbolKind = "function" | "keyword";

export function isWasmDocumentSymbol(
  value: unknown,
): value is WasmDocumentSymbol {
  if (!isObject(value) || !isWasmRange(value)) {
    return false;
  }
  const symbol = value as WasmRangeLike & Record<string, unknown>;
  return (
    hasSymbolName(symbol) && hasSymbolDetail(symbol) && hasSymbolKind(symbol)
  );
}

export function isWasmDefinitionLocation(
  value: unknown,
): value is WasmDefinitionLocation {
  if (!isObject(value) || !isWasmRange(value)) {
    return false;
  }
  const location = value as WasmRangeLike & Record<string, unknown>;
  return typeof location.uri === "string" && location.uri.length > 0;
}

export function isWasmWorkspaceSymbol(
  value: unknown,
): value is WasmWorkspaceSymbol {
  if (!isObject(value) || !isWasmDefinitionLocation(value)) {
    return false;
  }
  const symbol = value as WasmDefinitionLocation & Record<string, unknown>;
  return (
    hasSymbolName(symbol) &&
    hasSymbolKind(symbol) &&
    hasWorkspaceSymbolGroup(symbol) &&
    hasWorkspaceSymbolDeprecation(symbol)
  );
}

function hasSymbolName(value: Record<string, unknown>): boolean {
  return isNonEmptyString(value.name);
}

function hasSymbolDetail(value: Record<string, unknown>): boolean {
  return isOptionalNullableString(value.detail);
}

function hasSymbolKind(value: Record<string, unknown>): boolean {
  return isSymbolKind(value.kind);
}

function hasWorkspaceSymbolGroup(value: Record<string, unknown>): boolean {
  return isOptionalString(value.group);
}

function hasWorkspaceSymbolDeprecation(
  value: Record<string, unknown>,
): boolean {
  return typeof value.deprecated === "boolean";
}

function isSymbolKind(value: unknown): value is WasmSymbolKind {
  return value === "function" || value === "keyword";
}

function isOptionalString(value: unknown): boolean {
  return value === undefined || typeof value === "string";
}

function isOptionalNullableString(value: unknown): boolean {
  return value === undefined || value === null || typeof value === "string";
}
