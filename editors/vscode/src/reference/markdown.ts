import type { ReferenceEntry } from "./types";

export function renderReferenceDocumentMarkdown(
  entries: ReferenceEntry[],
): string {
  const groups = groupedEntries(entries);
  return `${[
    "# Scaffold Scheme Reference",
    "Generated from parsable Doc v2 forms such as `(doc ...)`, `(doc-next ...)`, `(extern-doc ...)`, `(moduledoc ...)`, and `(typedoc ...)`.",
    referenceStatsMarkdown(entries, groups.length),
    "## Contents",
    groups
      .map(
        (group) =>
          `- [${group.name}](#${anchor(group.name)}) (${group.entries.length})`,
      )
      .join("\n"),
    ...groups.flatMap((group) => [
      `## ${group.name}`,
      referenceGroupTableMarkdown(group.entries),
      ...group.entries.map(renderReferenceEntryMarkdown),
    ]),
  ].join("\n\n")}\n`;
}

export function renderReferenceEntryMarkdown(entry: ReferenceEntry): string {
  return `${referenceEntrySections(entry).join("\n\n")}\n`;
}

export function groupedEntries(entries: ReferenceEntry[]) {
  return [
    ...entries
      .reduce((groups, entry) => {
        groups.set(entry.group, [...(groups.get(entry.group) ?? []), entry]);
        return groups;
      }, new Map<string, ReferenceEntry[]>())
      .entries(),
  ]
    .sort(([left], [right]) => left.localeCompare(right))
    .map(([name, groupEntries]) => ({
      name,
      entries: groupEntries.sort((left, right) =>
        left.name.localeCompare(right.name),
      ),
    }));
}

function metadataMarkdown(entry: ReferenceEntry): string | undefined {
  const metadata = [
    `Group: ${entry.group}`,
    entry.since ? `Since: ${entry.since}` : undefined,
    entry.stability ? `Stability: ${entry.stability}` : undefined,
  ].filter((item): item is string => item !== undefined);
  if (metadata.length === 0) {
    return undefined;
  }
  return metadata.join("  \n");
}

function referenceEntrySections(entry: ReferenceEntry): string[] {
  return [
    `# ${entry.name}`,
    entry.summary ? `> ${entry.summary}` : undefined,
    metadataMarkdown(entry),
    entry.deprecated ? `> Deprecated: ${entry.deprecated}` : undefined,
    "## Signature",
    signatureMarkdown(entry),
    "## Parameters",
    paramsMarkdown(entry),
    entry.returns ? `## Returns\n\n${entry.returns}` : undefined,
    entry.markdown ? "## Details" : undefined,
    entry.markdown,
    "## Example",
    exampleMarkdown(entry),
    entry.see.length > 0 ? "## See Also" : undefined,
    seeAlsoMarkdown(entry),
    entry.source ? `## Source\n\n\`${entry.source}\`` : undefined,
  ].filter((part): part is string => Boolean(part));
}

function signatureMarkdown(entry: ReferenceEntry): string | undefined {
  if (!entry.signature) {
    return "_No signature documented._";
  }
  return `\`\`\`scheme\n${entry.signature}\n\`\`\``;
}

function paramsMarkdown(entry: ReferenceEntry): string | undefined {
  if (entry.params.length === 0) {
    return "_No parameters documented._";
  }
  return [
    "| Parameter | Description |",
    "| --- | --- |",
    ...entry.params.map(
      (param) => `| \`${param.name}\` | ${escapeTableCell(param.summary)} |`,
    ),
  ].join("\n");
}

function exampleMarkdown(entry: ReferenceEntry): string | undefined {
  if (!entry.example) {
    return "_No example documented._";
  }
  return `Example:\n\n\`\`\`scheme\n${entry.example}\n\`\`\``;
}

function seeAlsoMarkdown(entry: ReferenceEntry): string | undefined {
  if (entry.see.length === 0) {
    return undefined;
  }
  return `See also: ${entry.see.map((name) => `\`${name}\``).join(", ")}`;
}

function anchor(text: string): string {
  return text
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "");
}

function referenceStatsMarkdown(
  entries: ReferenceEntry[],
  groupCount: number,
): string {
  const deprecated = entries.filter((entry) => entry.deprecated).length;
  const documentedExamples = entries.filter((entry) => entry.example).length;
  return [
    "| Entries | Groups | Examples | Deprecated |",
    "| ---: | ---: | ---: | ---: |",
    `| ${entries.length} | ${groupCount} | ${documentedExamples} | ${deprecated} |`,
  ].join("\n");
}

function referenceGroupTableMarkdown(entries: ReferenceEntry[]): string {
  return [
    "| Name | Summary |",
    "| --- | --- |",
    ...entries.map((entry) => {
      const name = `[\`${entry.name}\`](#${anchor(entry.name)})`;
      return `| ${name} | ${escapeTableCell(entry.summary ?? "")} |`;
    }),
  ].join("\n");
}

function escapeTableCell(text: string): string {
  return text.replaceAll("|", "\\|").replace(/\s+/g, " ").trim();
}
