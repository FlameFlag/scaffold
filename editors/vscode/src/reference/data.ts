import { anchor } from "../../../../shared/reference-ids.js";
import { isNonNegativeInteger, isPositiveInteger } from "../wasm/number";
import { isObject } from "../wasm/object";
import { isNonEmptyString } from "../wasm/string";
import type { ReferenceEntry } from "./types";

const requiredStringFields = ["name", "group", "kind", "rendered_markdown"];
const nullableStringFields = [
  "signature",
  "summary",
  "markdown",
  "raw_markdown",
  "example",
  "returns",
  "effect",
  "stability",
  "since",
  "source",
  "source_location",
  "deprecated",
];
const requiredStringArrayFields = ["see", "requires_capability"];

type ReferenceEntryValidator = (value: Record<string, unknown>) => boolean;

const referenceEntryValidators: ReferenceEntryValidator[] = [
  hasRequiredStringFields,
  hasReferenceKind,
  hasReadableAnchors,
  hasNullableStringFields,
  hasStringArrayFields,
  hasReferenceParams,
  hasReferenceRange,
  hasHiddenFlag,
];

export function parseReferenceEntriesJson(text: string): ReferenceEntry[] {
  const value = JSON.parse(text) as unknown;

  if (!Array.isArray(value)) {
    throw new Error("expected reference entries to be an array");
  }

  return value.map((entry, index) => {
    if (!isReferenceEntry(entry)) {
      throw new Error(`invalid reference entry at index ${index}`);
    }
    return entry;
  });
}

function isReferenceEntry(value: unknown): value is ReferenceEntry {
  return (
    isObject(value) && referenceEntryValidators.every((valid) => valid(value))
  );
}

function hasRequiredStringFields(value: Record<string, unknown>): boolean {
  return requiredStringFields.every((field) => isNonEmptyString(value[field]));
}

function hasReferenceKind(value: Record<string, unknown>): boolean {
  return value.kind === "function" || value.kind === "keyword";
}

function hasReadableAnchors(value: Record<string, unknown>): boolean {
  return hasReadableAnchor(value.name) && hasReadableAnchor(value.group);
}

function hasNullableStringFields(value: Record<string, unknown>): boolean {
  return nullableStringFields.every((field) => isNullableString(value[field]));
}

function hasStringArrayFields(value: Record<string, unknown>): boolean {
  return requiredStringArrayFields.every((field) =>
    isStringArray(value[field]),
  );
}

function hasReferenceParams(value: Record<string, unknown>): boolean {
  return isReferenceParamArray(value.params);
}

function hasReferenceRange(value: Record<string, unknown>): boolean {
  return isNullableReferenceRange(value.range);
}

function hasHiddenFlag(value: Record<string, unknown>): boolean {
  return typeof value.hidden === "boolean";
}

function isNullableString(value: unknown): boolean {
  return typeof value === "string" || value === null;
}

function hasReadableAnchor(value: unknown): boolean {
  return typeof value === "string" && anchor(value).length > 0;
}

function isStringArray(value: unknown): boolean {
  return Array.isArray(value) && value.every((item) => isNonEmptyString(item));
}

function isReferenceParamArray(value: unknown): boolean {
  return (
    Array.isArray(value) &&
    value.every(
      (item) =>
        isObject(item) &&
        isNonEmptyString(item.name) &&
        typeof item.summary === "string",
    )
  );
}

function isNullableReferenceRange(value: unknown): boolean {
  return (
    value === null ||
    (isObject(value) &&
      isNonNegativeInteger(value.line) &&
      isNonNegativeInteger(value.start) &&
      isPositiveInteger(value.length))
  );
}
