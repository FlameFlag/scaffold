export type ReferenceDocument = {
  title: string;
  capabilities: ReferenceCapability[];
  catalog_schema: unknown;
  entries: ReferenceEntry[];
};

export type ReferenceCapability = {
  library_name: string[];
  library: string;
  bridge_library_name: string[];
  bridge_library: string;
  effect: string;
  modes: Record<string, string>;
  docs_source: string;
  notes: string;
};

export type ReferenceParam = {
  name: string;
  summary: string;
};

export type ReferenceRange = {
  line: number;
  start: number;
  length: number;
};

export type ReferenceEntry = {
  name: string;
  kind: "function" | "keyword";
  signature: string | null;
  summary: string | null;
  markdown: string | null;
  raw_markdown: string | null;
  rendered_markdown: string;
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
  source_location: string | null;
  range: ReferenceRange | null;
  hidden: boolean;
};
