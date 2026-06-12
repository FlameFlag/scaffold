export type ReferenceDocument = {
  title: string;
  capabilities: ReferenceCapability[];
  catalog_schema: unknown;
  entries: ReferenceEntry[];
};

export type ReferenceCapability = {
  library: string;
  effect: string;
  modes: Record<string, string>;
  notes: string;
};

export type ReferenceParam = {
  name: string;
  summary: string;
};

export type ReferenceEntry = {
  name: string;
  kind: "function" | "keyword";
  signature: string | null;
  summary: string | null;
  markdown: string | null;
  example: string | null;
  params: ReferenceParam[];
  returns: string | null;
  group: string;
  see: string[];
  effect: string | null;
  requires_capability: string[];
  stability: string | null;
  since: string | null;
  deprecated: string | null;
  source: string | null;
  range?: {
    line: number;
    start: number;
    length: number;
  };
  hidden?: boolean;
};

export function anchor(value: string) {
  return value
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "");
}
