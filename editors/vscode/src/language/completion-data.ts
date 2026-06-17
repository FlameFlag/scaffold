import { isObject } from "../wasm/object";
import { isNonEmptyString } from "../wasm/string";

export interface WasmCompletionItem {
  label: string;
  kind: "function" | "keyword";
  detail?: string;
  documentation: string;
  insert_text: string;
  insert_text_is_snippet: boolean;
  sort_text: string;
  deprecated: boolean;
}

const requiredStringFields = ["documentation", "sort_text"];
const requiredNonEmptyStringFields = ["label", "insert_text"];
const requiredBooleanFields = ["insert_text_is_snippet", "deprecated"];

export function isWasmCompletionItem(
  item: unknown,
): item is WasmCompletionItem {
  return (
    isObject(item) &&
    requiredStringFields.every((field) => typeof item[field] === "string") &&
    requiredNonEmptyStringFields.every((field) =>
      isNonEmptyString(item[field]),
    ) &&
    requiredBooleanFields.every((field) => typeof item[field] === "boolean") &&
    isCompletionKind(item.kind) &&
    isOptionalString(item.detail)
  );
}

function isCompletionKind(value: unknown): value is "function" | "keyword" {
  return value === "function" || value === "keyword";
}

function isOptionalString(value: unknown): boolean {
  return value === undefined || typeof value === "string";
}
