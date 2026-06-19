import { markdownTable } from "markdown-table";

import { entryId, groupId } from "../../../../shared/reference-ids.js";
import type { ReferenceEntry } from "./types";

export function renderReferenceDocumentMarkdown(
  entries: ReferenceEntry[],
): string {
  const groups = groupedEntries(entries);
  return `${[
    "# Scaffold Scheme Reference",
    "Generated from parsable documentation forms such as `(doc ...)`, `(doc-next ...)`, `(extern-doc ...)`, `(moduledoc ...)`, and `(typedoc ...)`.",
    referenceStatsMarkdown(entries, groups.length),
    "## Contents",
    groups
      .map(
        (group) =>
          `- [${markdownText(group.name)}](#${groupId(group.name)}) (${group.entries.length})`,
      )
      .join("\n"),
    ...groups.flatMap((group) => [
      anchoredHeading(2, markdownText(group.name), groupId(group.name)),
      referenceGroupTableMarkdown(group.entries),
      ...group.entries.map((entry) =>
        renderReferenceEntryMarkdown(entry, { headingLevel: 3 }),
      ),
    ]),
  ].join("\n\n")}\n`;
}

type ReferenceEntryMarkdownOptions = {
  headingLevel?: 1 | 2 | 3;
};

export function renderReferenceEntryMarkdown(
  entry: ReferenceEntry,
  options: ReferenceEntryMarkdownOptions = {},
): string {
  const headingLevel = options.headingLevel ?? 1;
  const sourceHeading = "#".repeat(Math.min(headingLevel + 1, 6));
  return `${[
    anchoredHeading(
      headingLevel,
      markdownCodeSpan(entry.name),
      entryId(entry.name),
    ),
    entry.rendered_markdown.trim(),
    sourceMarkdown(entry, sourceHeading),
  ]
    .filter((part): part is string => Boolean(part))
    .join("\n\n")}\n`;
}

export function renderMissingReferenceEntryMarkdown(name?: string): string {
  const detail = name
    ? `Reference entry ${markdownCodeSpan(name)} was not found.`
    : "Reference entry not found.";

  return `# Scaffold Scheme Reference\n\n${detail}\n`;
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

function sourceMarkdown(
  entry: ReferenceEntry,
  heading = "##",
): string | undefined {
  const source = entry.source_location ?? entry.source;
  return source
    ? `${heading} Source\n\n${referenceTableMarkdown([
        ["Field", "Value"],
        ["Source", markdownCodeSpan(source)],
      ])}`
    : undefined;
}

function anchoredHeading(
  level: 1 | 2 | 3,
  markdown: string,
  id: string,
): string {
  return `${"#".repeat(level)} <a id="${id}"></a>${markdown}`;
}

function referenceStatsMarkdown(
  entries: ReferenceEntry[],
  groupCount: number,
): string {
  const deprecated = entries.filter((entry) => entry.deprecated).length;
  const documentedExamples = entries.filter((entry) => entry.example).length;
  return referenceTableMarkdown(
    [
      ["Entries", "Groups", "Examples", "Deprecated"],
      [
        String(entries.length),
        String(groupCount),
        String(documentedExamples),
        String(deprecated),
      ],
    ],
    { align: ["r", "r", "r", "r"] },
  );
}

function referenceGroupTableMarkdown(entries: ReferenceEntry[]): string {
  return referenceTableMarkdown([
    ["Name", "Summary"],
    ...entries.map((entry) => {
      const name = `[${markdownCodeSpan(entry.name)}](#${entryId(entry.name)})`;
      return [name, entry.summary ?? ""];
    }),
  ]);
}

function referenceTableMarkdown(
  rows: string[][],
  options?: Parameters<typeof markdownTable>[1],
): string {
  return markdownTable(
    rows.map((row) => row.map(tableCell)),
    {
      alignDelimiters: true,
      ...options,
    },
  );
}

function tableCell(text: string): string {
  return text
    .trim()
    .replaceAll("\r\n", "\n")
    .replaceAll("\r", "\n")
    .replaceAll("|", "\\|")
    .replaceAll("\n", "<br>");
}

export function markdownCodeSpan(value: string): string {
  const delimiter = "`".repeat(maxConsecutiveBackticks(value) + 1);
  if (value.includes("`") || value.startsWith(" ") || value.endsWith(" ")) {
    return `${delimiter} ${value} ${delimiter}`;
  }
  return `${delimiter}${value}${delimiter}`;
}

function markdownText(value: string): string {
  return value
    .replaceAll("\r\n", "\n")
    .replaceAll("\r", "\n")
    .replaceAll("\n", " ")
    .replace(/[\\`*_{}[\]<>()#+\-.!]/g, "\\$&");
}

function maxConsecutiveBackticks(value: string): number {
  let maxRun = 0;
  let run = 0;
  for (const character of value) {
    if (character === "`") {
      run += 1;
      maxRun = Math.max(maxRun, run);
    } else {
      run = 0;
    }
  }
  return maxRun;
}
