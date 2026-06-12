export interface ReferenceEntry {
  name: string;
  signature?: string;
  summary?: string;
  markdown?: string;
  example?: string;
  params: ReferenceParam[];
  returns?: string;
  group: string;
  see: string[];
  stability?: string;
  since?: string;
  deprecated?: string;
  source?: string;
}

export interface ReferenceParam {
  name: string;
  summary: string;
}

export type ReferenceNode = ReferenceGroupNode | ReferenceEntryNode;

export interface ReferenceGroupNode {
  kind: "group";
  name: string;
  entries: ReferenceEntry[];
}

export interface ReferenceEntryNode {
  kind: "entry";
  entry: ReferenceEntry;
}
