import type {
  ReferenceDocument as RawReferenceDocument,
  ReferenceEntry as RawReferenceEntry,
  ReferenceCapability,
} from "../../shared/reference";
import { anchor, entryId, groupId } from "../../shared/reference-ids.js";
import { assertSafeRenderedHtml } from "./rendered-html";

export type { ReferenceCapability };

export type RenderedReferenceParam = {
  name: string;
  summaryHtml: string;
};

export type ReferenceSourceSnippet = {
  label: string;
  code: string;
  html: string;
  startLine: number;
};

export type RenderedReferenceEntry = {
  rawMarkdownHtml: string | null;
  params: RenderedReferenceParam[];
  returnsHtml: string | null;
  sourceSnippet: ReferenceSourceSnippet | null;
};

export type ReferenceEntry = RawReferenceEntry & {
  rendered: RenderedReferenceEntry;
};

export type ReferenceDocument = Omit<RawReferenceDocument, "entries"> & {
  entries: ReferenceEntry[];
};

export function parseReferenceDocument(value: unknown): ReferenceDocument {
  if (!isObject(value)) {
    throw new Error("expected reference document to be an object");
  }

  if (typeof value.title !== "string") {
    throw new Error("expected reference document title to be a string");
  }

  if (!Array.isArray(value.capabilities)) {
    throw new Error("expected reference capabilities to be an array");
  }

  if (!Array.isArray(value.entries)) {
    throw new Error("expected reference entries to be an array");
  }

  const capabilities = value.capabilities.map(parseReferenceCapability);
  const entries = value.entries.map(parseReferenceEntry);
  assertUniqueValues(
    capabilities.map((capability) => capability.library),
    "capability library",
  );
  assertUniqueValues(
    entries.map((entry) => entry.name),
    "reference entry name",
  );
  assertNonEmptyDerivedValues(
    entries.map((entry) => entry.name),
    "reference entry anchor",
    anchor,
  );
  assertUniqueDerivedValues(
    entries.map((entry) => entry.name),
    "reference entry id",
    entryId,
  );
  assertUniqueDerivedValues(
    [...new Set(entries.map((entry) => entry.group))],
    "reference group id",
    groupId,
  );

  return {
    title: value.title,
    capabilities,
    catalog_schema: value.catalog_schema,
    entries,
  };
}

export { entryId, groupId };

export function targetIdFromHash(hash: string): string | null {
  const fragment = hash.startsWith("#") ? hash.slice(1) : hash;
  if (fragment.length === 0) {
    return null;
  }

  try {
    return decodeURIComponent(fragment);
  } catch {
    return fragment;
  }
}

function parseReferenceCapability(
  value: unknown,
  index: number,
): ReferenceCapability {
  if (!isObject(value)) {
    throw new Error(`expected capability ${index} to be an object`);
  }

  const modes = value.modes;
  if (!isStringRecord(modes)) {
    throw new Error(`expected capability ${index} modes to be string values`);
  }

  return {
    library_name: requiredStringArray(value, "library_name", label(index)),
    library: requiredNonEmptyString(value, "library", label(index)),
    bridge_library_name: requiredStringArray(
      value,
      "bridge_library_name",
      label(index),
    ),
    bridge_library: requiredString(value, "bridge_library", label(index)),
    effect: requiredString(value, "effect", label(index)),
    modes,
    docs_source: requiredString(value, "docs_source", label(index)),
    notes: requiredString(value, "notes", label(index)),
  };
}

function parseReferenceEntry(value: unknown, index: number): ReferenceEntry {
  if (!isObject(value)) {
    throw new Error(`expected entry ${index} to be an object`);
  }

  const name = requiredNonEmptyString(value, "name", `entry ${index}`);
  const entryLabel = `entry ${index} (${name})`;

  return {
    name,
    kind: requiredReferenceKind(value, entryLabel),
    signature: optionalString(value, "signature", entryLabel),
    summary: optionalString(value, "summary", entryLabel),
    markdown: optionalString(value, "markdown", entryLabel),
    raw_markdown: optionalString(value, "raw_markdown", entryLabel),
    rendered_markdown: requiredNonEmptyString(
      value,
      "rendered_markdown",
      entryLabel,
    ),
    example: optionalString(value, "example", entryLabel),
    params: requiredArray(value, "params", entryLabel).map(
      (param, paramIndex) =>
        parseReferenceParam(param, `${entryLabel} param ${paramIndex}`),
    ),
    returns: optionalString(value, "returns", entryLabel),
    group: requiredNonEmptyString(value, "group", entryLabel),
    see: requiredStringArray(value, "see", entryLabel),
    effect: optionalString(value, "effect", entryLabel),
    requires_capability: requiredStringArray(
      value,
      "requires_capability",
      entryLabel,
    ),
    stability: optionalString(value, "stability", entryLabel),
    since: optionalString(value, "since", entryLabel),
    deprecated: optionalString(value, "deprecated", entryLabel),
    source: optionalString(value, "source", entryLabel),
    source_location: optionalString(value, "source_location", entryLabel),
    range: optionalReferenceRange(value.range, entryLabel),
    hidden: requiredBoolean(value, "hidden", entryLabel),
    rendered: parseRenderedReferenceEntry(
      value.rendered,
      `${entryLabel} rendered data`,
    ),
  };
}

function parseReferenceParam(
  value: unknown,
  label: string,
): RawReferenceEntry["params"][number] {
  if (!isObject(value)) {
    throw new Error(`expected ${label} to be an object`);
  }

  return {
    name: requiredNonEmptyString(value, "name", label),
    summary: requiredString(value, "summary", label),
  };
}

function parseRenderedReferenceEntry(
  value: unknown,
  label: string,
): RenderedReferenceEntry {
  if (!isObject(value)) {
    throw new Error(`expected ${label} rendered data to be an object`);
  }

  return {
    rawMarkdownHtml: optionalHtml(value, "rawMarkdownHtml", label),
    params: requiredArray(value, "params", label).map((param, paramIndex) =>
      parseRenderedReferenceParam(
        param,
        `${label} rendered param ${paramIndex}`,
      ),
    ),
    returnsHtml: optionalHtml(value, "returnsHtml", label),
    sourceSnippet: optionalSourceSnippet(value.sourceSnippet, label),
  };
}

function parseRenderedReferenceParam(
  value: unknown,
  label: string,
): RenderedReferenceParam {
  if (!isObject(value)) {
    throw new Error(`expected ${label} to be an object`);
  }

  return {
    name: requiredNonEmptyString(value, "name", label),
    summaryHtml: requiredHtml(value, "summaryHtml", label),
  };
}

function optionalSourceSnippet(
  value: unknown,
  label: string,
): ReferenceSourceSnippet | null {
  if (value === null) {
    return null;
  }

  if (!isObject(value)) {
    throw new Error(`expected ${label} source snippet to be null or an object`);
  }

  return {
    label: requiredString(value, "label", `${label} source snippet`),
    code: requiredString(value, "code", `${label} source snippet`),
    html: requiredHtml(value, "html", `${label} source snippet`),
    startLine: requiredPositiveInteger(
      value,
      "startLine",
      `${label} source snippet`,
    ),
  };
}

function optionalReferenceRange(
  value: unknown,
  label: string,
): RawReferenceEntry["range"] {
  if (value === null) {
    return null;
  }

  if (!isObject(value)) {
    throw new Error(`expected ${label} range to be null or an object`);
  }

  return {
    line: requiredNonNegativeInteger(value, "line", `${label} range`),
    start: requiredNonNegativeInteger(value, "start", `${label} range`),
    length: requiredPositiveInteger(value, "length", `${label} range`),
  };
}

function requiredReferenceKind(
  value: Record<string, unknown>,
  label: string,
): RawReferenceEntry["kind"] {
  if (value.kind === "function" || value.kind === "keyword") {
    return value.kind;
  }

  throw new Error(`expected ${label} kind to be function or keyword`);
}

function requiredString(
  value: Record<string, unknown>,
  field: string,
  label: string,
): string {
  if (typeof value[field] === "string") {
    return value[field];
  }

  throw new Error(`expected ${label}.${field} to be a string`);
}

function requiredNonEmptyString(
  value: Record<string, unknown>,
  field: string,
  label: string,
): string {
  const text = requiredString(value, field, label);
  if (text.trim().length === 0) {
    throw new Error(`expected ${label}.${field} to be non-empty`);
  }
  return text;
}

function optionalString(
  value: Record<string, unknown>,
  field: string,
  label: string,
): string | null {
  if (value[field] === null || typeof value[field] === "string") {
    return value[field];
  }

  throw new Error(`expected ${label}.${field} to be null or a string`);
}

function optionalHtml(
  value: Record<string, unknown>,
  field: string,
  label: string,
): string | null {
  const html = optionalString(value, field, label);
  if (html !== null) {
    assertSafeRenderedHtml(html, `${label}.${field}`);
  }
  return html;
}

function requiredHtml(
  value: Record<string, unknown>,
  field: string,
  label: string,
): string {
  const html = requiredString(value, field, label);
  assertSafeRenderedHtml(html, `${label}.${field}`);
  return html;
}

function requiredBoolean(
  value: Record<string, unknown>,
  field: string,
  label: string,
): boolean {
  if (typeof value[field] === "boolean") {
    return value[field];
  }

  throw new Error(`expected ${label}.${field} to be a boolean`);
}

function requiredArray(
  value: Record<string, unknown>,
  field: string,
  label: string,
): unknown[] {
  if (Array.isArray(value[field])) {
    return value[field];
  }

  throw new Error(`expected ${label}.${field} to be an array`);
}

function requiredStringArray(
  value: Record<string, unknown>,
  field: string,
  label: string,
): string[] {
  const array = requiredArray(value, field, label);
  if (array.every((item) => isNonEmptyString(item))) {
    return array;
  }

  throw new Error(
    `expected ${label}.${field} to contain only non-empty strings`,
  );
}

function isNonEmptyString(value: unknown): value is string {
  return typeof value === "string" && value.trim().length > 0;
}

function requiredNonNegativeInteger(
  value: Record<string, unknown>,
  field: string,
  label: string,
): number {
  if (isNonNegativeInteger(value[field])) {
    return value[field];
  }

  throw new Error(`expected ${label}.${field} to be a non-negative integer`);
}

function requiredPositiveInteger(
  value: Record<string, unknown>,
  field: string,
  label: string,
): number {
  const fieldValue = value[field];
  if (
    typeof fieldValue === "number" &&
    Number.isInteger(fieldValue) &&
    fieldValue > 0
  ) {
    return fieldValue;
  }

  throw new Error(`expected ${label}.${field} to be a positive integer`);
}

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isStringRecord(value: unknown): value is Record<string, string> {
  return (
    isObject(value) &&
    Object.values(value).every((item) => typeof item === "string")
  );
}

function isNonNegativeInteger(value: unknown): value is number {
  return typeof value === "number" && Number.isInteger(value) && value >= 0;
}

function label(index: number) {
  return `capability ${index}`;
}

function assertUniqueValues(values: string[], label: string): void {
  const seen = new Set<string>();
  for (const value of values) {
    if (seen.has(value)) {
      throw new Error(`duplicate ${label}: ${value}`);
    }
    seen.add(value);
  }
}

function assertNonEmptyDerivedValues(
  values: string[],
  label: string,
  derive: (value: string) => string,
): void {
  for (const value of values) {
    const derived = derive(value);
    if (derived.length === 0 || derived.endsWith("-")) {
      throw new Error(`empty ${label}: ${value}`);
    }
  }
}

function assertUniqueDerivedValues(
  values: string[],
  label: string,
  derive: (value: string) => string,
): void {
  const seen = new Map<string, string>();
  for (const value of values) {
    const derived = derive(value);
    if (derived.length === 0 || derived.endsWith("-")) {
      throw new Error(`empty ${label}: ${value}`);
    }
    const previous = seen.get(derived);
    if (previous !== undefined) {
      throw new Error(
        `duplicate ${label}: ${derived} from ${previous} and ${value}`,
      );
    }
    seen.set(derived, value);
  }
}
