import type { ReferenceEntry } from "../../../../shared/reference";

export type { ReferenceEntry };

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
