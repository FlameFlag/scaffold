import { isWasmCompletionItem } from "../src/language/completion-data.ts";
import {
  diagnosticLineNumber,
  isWasmDiagnostic,
  missingDocCode,
} from "../src/language/diagnostics-data.ts";
import { isWasmInlayHint } from "../src/language/inlay-hints-data.ts";
import {
  isWasmDefinitionLocation,
  isWasmDocumentSymbol,
  isWasmWorkspaceSymbol,
} from "../src/language/navigation-data.ts";
import { isWasmSemanticToken } from "../src/language/semantic-tokens-data.ts";
import {
  isFormContext,
  isWasmSignatureHelp,
} from "../src/language/signature-help-data.ts";
import { parseReferenceEntriesJson } from "../src/reference/data.ts";
import {
  groupedEntries,
  markdownCodeSpan,
  renderMissingReferenceEntryMarkdown,
  renderReferenceDocumentMarkdown,
  renderReferenceEntryMarkdown,
} from "../src/reference/markdown.ts";
import type { ReferenceEntry } from "../src/reference/types.ts";
import {
  referenceEntryNameFromPath,
  referenceEntryPath,
} from "../src/reference/uri.ts";
import {
  parseBundledSources,
  shouldOpenSourceBundleUri,
  sourceBundleContent,
  sourceBundleLoadFailure,
} from "../src/source/bundle.ts";
import { parseWasmJson } from "../src/wasm/json.ts";
import { isObject } from "../src/wasm/object.ts";
import { isWasmPosition } from "../src/wasm/position.ts";
import { isWasmRange } from "../src/wasm/range.ts";

const catalogTool = referenceEntry({
  name: "catalog/tool",
  group: "Catalog",
  summary: "Create a raw catalog tool | helper.",
  rendered_markdown:
    "```scheme\n(catalog/tool name action field ...)\n```\n\n| Field | Value |\n| --- | --- |\n| Group | Catalog |",
  source: "src/dsl/std/catalog/tool.scm",
  source_location: "src/dsl/std/catalog/tool.scm:12",
});
const pathTool = referenceEntry({
  name: "path/join",
  group: "Paths",
  summary: "Join path | parts.\r\nKeep  spacing.",
  rendered_markdown:
    "Join path parts.\n\n| Parameter | Description |\n| --- | --- |\n| `part` | Path part. |",
});

const grouped = groupedEntries([pathTool, catalogTool]);
assertEqual(grouped.map((group) => group.name).join(","), "Catalog,Paths");

const encodedPath = referenceEntryPath("catalog/tool");
assertEqual(encodedPath, "/entries/catalog%2Ftool.md");
assertEqual(referenceEntryNameFromPath(encodedPath), "catalog/tool");
assertEqual(referenceEntryNameFromPath("/entries/%E0%A4%A.md"), undefined);
assertEqual(referenceEntryNameFromPath("/reference.md"), undefined);
assertEqual(
  parseReferenceEntriesJson(JSON.stringify([catalogTool]))[0].name,
  "catalog/tool",
);
assertThrows(
  () => parseReferenceEntriesJson("{}"),
  "expected reference entries to be an array",
);
assertThrows(
  () => parseReferenceEntriesJson('[{"name":"broken"}]'),
  "invalid reference entry at index 0",
);
assertThrows(() => {
  const entry = referenceEntry({});
  delete (entry as Partial<ReferenceEntry>).hidden;
  parseReferenceEntriesJson(JSON.stringify([entry]));
}, "invalid reference entry at index 0");
assertThrows(
  () =>
    parseReferenceEntriesJson(
      JSON.stringify([referenceEntry({ params: [{ name: "path" } as never] })]),
    ),
  "invalid reference entry at index 0",
);
assertThrows(
  () =>
    parseReferenceEntriesJson(
      JSON.stringify([
        referenceEntry({ params: [{ name: " ", summary: "Path." }] }),
      ]),
    ),
  "invalid reference entry at index 0",
);
assertThrows(
  () =>
    parseReferenceEntriesJson(JSON.stringify([referenceEntry({ see: [" "] })])),
  "invalid reference entry at index 0",
);
assertThrows(
  () =>
    parseReferenceEntriesJson(
      JSON.stringify([
        referenceEntry({ range: { line: 0, start: 0, length: 0 } }),
      ]),
    ),
  "invalid reference entry at index 0",
);
assertThrows(
  () =>
    parseReferenceEntriesJson(
      JSON.stringify([referenceEntry({ kind: "macro" as never })]),
    ),
  "invalid reference entry at index 0",
);
assertThrows(
  () =>
    parseReferenceEntriesJson(
      JSON.stringify([referenceEntry({ name: "???" })]),
    ),
  "invalid reference entry at index 0",
);
assertThrows(
  () =>
    parseReferenceEntriesJson(
      JSON.stringify([referenceEntry({ group: "..." })]),
    ),
  "invalid reference entry at index 0",
);
assertEqual(
  isWasmCompletionItem({
    label: "tool",
    kind: "function",
    documentation: "Create a tool.",
    insert_text: "(tool name)",
    insert_text_is_snippet: true,
    sort_text: "tool",
    deprecated: false,
  }),
  true,
);
assertEqual(
  isWasmCompletionItem({
    label: "broken",
    kind: "macro",
    documentation: "Broken item.",
    insert_text: "broken",
    insert_text_is_snippet: false,
    sort_text: "broken",
    deprecated: false,
  }),
  false,
);
assertEqual(
  isWasmCompletionItem({
    label: " ",
    kind: "function",
    documentation: "Broken item.",
    insert_text: "broken",
    insert_text_is_snippet: false,
    sort_text: "broken",
    deprecated: false,
  }),
  false,
);
assertEqual(
  isWasmCompletionItem({
    label: "broken",
    kind: "function",
    documentation: "Broken item.",
    insert_text: "",
    insert_text_is_snippet: false,
    sort_text: "broken",
    deprecated: false,
  }),
  false,
);
assertEqual(isFormContext({ head: "tool", active_argument: 1 }), true);
assertEqual(isFormContext({ head: " ", active_argument: 1 }), false);
assertEqual(isFormContext({ head: "tool", active_argument: -1 }), false);
assertEqual(
  isWasmSignatureHelp({
    label: "(tool name)",
    documentation: "Create a tool.",
    parameters: [{ label: "name", documentation: "Tool name." }],
  }),
  true,
);
assertEqual(
  isWasmSignatureHelp({
    label: "(tool name)",
    documentation: "Create a tool.",
    parameters: [{ label: 1 }],
  }),
  false,
);
assertEqual(
  isWasmSignatureHelp({
    label: " ",
    documentation: "Create a tool.",
    parameters: [{ label: "name", documentation: "Tool name." }],
  }),
  false,
);
assertEqual(
  isWasmSignatureHelp({
    label: "(tool name)",
    documentation: "Create a tool.",
    parameters: [{ label: " " }],
  }),
  false,
);
assertEqual(
  isWasmDiagnostic({
    message: "Missing docs.",
    offset: 1,
    length: 4,
    severity: "warning",
    code: missingDocCode,
    data: { name: "tool", line: 2 },
  }),
  true,
);
assertEqual(
  isWasmDiagnostic({
    message: "Bad diagnostic.",
    offset: 1,
    length: 4,
    severity: "severe",
    code: missingDocCode,
  }),
  false,
);
assertEqual(
  isWasmDiagnostic({
    message: "Bad diagnostic data.",
    offset: 1,
    length: 4,
    severity: "warning",
    code: missingDocCode,
    data: { name: "tool", line: -1 },
  }),
  false,
);
assertEqual(diagnosticLineNumber({ line: 2 }, 0, 4), 2);
assertEqual(diagnosticLineNumber({ line: 9 }, 1, 4), undefined);
assertEqual(diagnosticLineNumber(undefined, 1, 4), 1);
assertEqual(diagnosticLineNumber(undefined, 4, 4), undefined);
assertEqual(
  isWasmInlayHint({ line: 1, start: 2, label: "name:", tooltip: "Tool name." }),
  true,
);
assertEqual(
  isWasmInlayHint({ line: 1, start: 2, label: " ", tooltip: "Tool name." }),
  false,
);
assertEqual(
  isWasmInlayHint({
    line: 1,
    start: -1,
    label: "name:",
    tooltip: "Tool name.",
  }),
  false,
);
assertEqual(
  isWasmDocumentSymbol({
    line: 1,
    start: 2,
    length: 4,
    name: "tool",
    detail: null,
    kind: "function",
  }),
  true,
);
assertEqual(
  isWasmDocumentSymbol({
    line: 1,
    start: 2,
    length: 4,
    name: "tool",
    detail: 12,
    kind: "function",
  }),
  false,
);
assertEqual(
  isWasmDocumentSymbol({
    line: 1,
    start: 2,
    length: 4,
    name: "   ",
    detail: null,
    kind: "function",
  }),
  false,
);
assertEqual(
  isWasmDefinitionLocation({
    line: 1,
    start: 2,
    length: 4,
    uri: "scaffold-source:/std/catalog/tool.scm",
  }),
  true,
);
assertEqual(
  isWasmWorkspaceSymbol({
    line: 1,
    start: 2,
    length: 4,
    uri: "scaffold-source:/std/catalog/tool.scm",
    name: "tool",
    kind: "macro",
    deprecated: false,
  }),
  false,
);
assertEqual(
  isWasmWorkspaceSymbol({
    line: 1,
    start: 2,
    length: 4,
    uri: "scaffold-source:/std/catalog/tool.scm",
    name: "",
    kind: "function",
    deprecated: false,
  }),
  false,
);
assertEqual(
  isWasmSemanticToken({
    text: "tool",
    line: 1,
    start: 2,
    length: 4,
    token_type: "function",
    modifiers: ["defaultLibrary"],
  }),
  true,
);
assertEqual(
  isWasmSemanticToken({
    text: "tool",
    line: 1,
    start: 2,
    length: 4,
    token_type: "function",
    modifiers: ["unknown"],
  }),
  false,
);
assertEqual(
  isWasmSemanticToken({
    text: " ",
    line: 1,
    start: 2,
    length: 4,
    token_type: "function",
    modifiers: [],
  }),
  false,
);
assertEqual(JSON.stringify(parseWasmJson("[1,2]", [])), "[1,2]");
assertEqual(
  JSON.stringify(parseWasmJson("not json", ["fallback"])),
  '["fallback"]',
);
assertEqual(isWasmRange({ line: 0, start: 0, length: 1 }), true);
assertEqual(isWasmRange({ line: -1, start: 0, length: 1 }), false);
assertEqual(isWasmRange({ line: 0, start: 0, length: 0 }), false);
assertEqual(isWasmPosition({ line: 0, start: 0 }), true);
assertEqual(isWasmPosition({ line: 0, start: -1 }), false);
assertEqual(isObject({}), true);
assertEqual(isObject([]), false);
const parsedSources = parseBundledSources(
  '{"src/dsl/std/catalog/tool.scm":"(define tool)"}',
);
assertEqual(parsedSources["src/dsl/std/catalog/tool.scm"], "(define tool)");
assertThrows(
  () => parseBundledSources("[]"),
  "expected source bundle to be an object",
);
const failedSourceBundle = sourceBundleLoadFailure(new Error("broken json"));
assertEqual(
  shouldOpenSourceBundleUri(failedSourceBundle, "src/dsl/std/catalog/tool.scm"),
  true,
);
assertIncludes(
  sourceBundleContent(failedSourceBundle, "src/dsl/std/catalog/tool.scm"),
  "source bundle could not be loaded",
);
assertIncludes(
  sourceBundleContent({ sources: {}, error: null }, "missing.scm"),
  "source not found",
);
const inheritedSources = Object.create({
  "src/dsl/std/catalog/tool.scm": "(define inherited)",
});
assertEqual(
  shouldOpenSourceBundleUri(
    { sources: inheritedSources, error: null },
    "src/dsl/std/catalog/tool.scm",
  ),
  false,
);
assertIncludes(
  sourceBundleContent(
    { sources: inheritedSources, error: null },
    "src/dsl/std/catalog/tool.scm",
  ),
  "source not found",
);

const entryMarkdown = renderReferenceEntryMarkdown(catalogTool);
assertIncludes(entryMarkdown, '# <a id="entry-catalog-tool-');
assertIncludes(
  entryMarkdown,
  "```scheme\n(catalog/tool name action field ...)\n```",
);
assertIncludes(entryMarkdown, "## Source");
assertIncludes(entryMarkdown, "`src/dsl/std/catalog/tool.scm:12`");
assertNotIncludes(entryMarkdown, "# catalog/tool\n");
assertNotIncludes(entryMarkdown, "markdownHtml");

const backtickEntryMarkdown = renderReferenceEntryMarkdown(
  referenceEntry({
    name: "bad`name",
    group: "Fixtures",
    source: "src/dsl/std/bad`source.scm",
    source_location: "src/dsl/std/bad`source.scm:4",
  }),
);
assertIncludes(backtickEntryMarkdown, '# <a id="entry-bad-name-');
assertIncludes(backtickEntryMarkdown, "`` bad`name ``");
assertIncludes(backtickEntryMarkdown, "`` src/dsl/std/bad`source.scm:4 ``");
assertNotIncludes(backtickEntryMarkdown, "# bad`name\n");
assertNotIncludes(backtickEntryMarkdown, "`src/dsl/std/bad`source.scm:4`");

const documentMarkdown = renderReferenceDocumentMarkdown([
  pathTool,
  catalogTool,
]);
assertIncludes(documentMarkdown, "Generated from parsable documentation forms");
assertNotIncludes(documentMarkdown, "Doc v2 forms");
assertIncludes(documentMarkdown, "- [Catalog](#group-catalog) (1)");
assertIncludes(documentMarkdown, "[`catalog/tool`](#entry-catalog-tool-");
assertIncludes(documentMarkdown, "Create a raw catalog tool \\| helper.");
assertIncludes(documentMarkdown, "Join path \\| parts.<br>Keep  spacing.");
assertIncludes(documentMarkdown, '## <a id="group-catalog"></a>Catalog');
assertIncludes(documentMarkdown, '### <a id="entry-catalog-tool-');
assertIncludes(documentMarkdown, '### <a id="entry-path-join-');
assertNotIncludes(documentMarkdown, "### catalog/tool");
assertEqual(markdownCodeSpan("catalog/tool"), "`catalog/tool`");
assertEqual(markdownCodeSpan("bad`name"), "`` bad`name ``");
assertEqual(markdownCodeSpan("bad``name"), "``` bad``name ```");
assertEqual(markdownCodeSpan(" padded "), "`  padded  `");
assertEqual(
  renderMissingReferenceEntryMarkdown("catalog/tool"),
  "# Scaffold Scheme Reference\n\nReference entry `catalog/tool` was not found.\n",
);
assertEqual(
  renderMissingReferenceEntryMarkdown("bad`name"),
  "# Scaffold Scheme Reference\n\nReference entry `` bad`name `` was not found.\n",
);
assertEqual(
  renderMissingReferenceEntryMarkdown(),
  "# Scaffold Scheme Reference\n\nReference entry not found.\n",
);

const backtickDocumentMarkdown = renderReferenceDocumentMarkdown([
  referenceEntry({ name: "bad`name", group: "Fixtures" }),
]);
assertIncludes(backtickDocumentMarkdown, "[`` bad`name ``](#entry-bad-name-");
assertIncludes(backtickDocumentMarkdown, '### <a id="entry-bad-name-');
assertNotIncludes(backtickDocumentMarkdown, "[`bad`name`]");
assertNotIncludes(backtickDocumentMarkdown, "### bad`name");

const escapedGroupDocumentMarkdown = renderReferenceDocumentMarkdown([
  referenceEntry({ name: "group/entry", group: "Bad [Group] | Plus+" }),
]);
assertIncludes(
  escapedGroupDocumentMarkdown,
  "- [Bad \\[Group\\] | Plus\\+](#group-bad-group-plus) (1)",
);
assertIncludes(
  escapedGroupDocumentMarkdown,
  '## <a id="group-bad-group-plus"></a>Bad \\[Group\\] | Plus\\+',
);
assertNotIncludes(escapedGroupDocumentMarkdown, "- [Bad [Group] | Plus+]");

const collidingAnchorMarkdown = renderReferenceDocumentMarkdown([
  referenceEntry({ name: "tool/path", group: "Tools" }),
  referenceEntry({ name: "tool-path", group: "Tools" }),
]);
const collidingEntryAnchors = [
  ...collidingAnchorMarkdown.matchAll(/id="(entry-tool-path-[^"]+)"/g),
].map((match) => match[1]);
assertEqual(String(new Set(collidingEntryAnchors).size), "2");

console.log("reference markdown checks passed");

function referenceEntry(overrides: Partial<ReferenceEntry>): ReferenceEntry {
  return {
    name: "fixture",
    kind: "function",
    signature: null,
    summary: null,
    markdown: null,
    raw_markdown: null,
    rendered_markdown: "Fixture docs.",
    example: null,
    params: [],
    returns: null,
    group: "Fixtures",
    see: [],
    effect: null,
    requires_capability: [],
    stability: null,
    since: null,
    deprecated: null,
    source: null,
    source_location: null,
    range: null,
    hidden: false,
    ...overrides,
  };
}

function assertIncludes(value: string, expected: string): void {
  if (!value.includes(expected)) {
    throw new Error(`Expected markdown to include ${JSON.stringify(expected)}`);
  }
}

function assertNotIncludes(value: string, expected: string): void {
  if (value.includes(expected)) {
    throw new Error(`Expected markdown to omit ${JSON.stringify(expected)}`);
  }
}

function assertThrows(fn: () => void, expected: string): void {
  try {
    fn();
  } catch (error) {
    assertIncludes(errorMessage(error), expected);
    return;
  }

  throw new Error(`Expected function to throw ${JSON.stringify(expected)}`);
}

function assertEqual(actual: unknown, expected: unknown): void {
  if (actual !== expected) {
    throw new Error(
      `Expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`,
    );
  }
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
