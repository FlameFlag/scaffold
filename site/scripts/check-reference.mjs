import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import {
  entryId,
  groupId,
  parseReferenceDocument,
  targetIdFromHash,
} from "../src/reference.ts";
import { createReferenceSearchIndex } from "../src/reference-search.ts";
import { containsUnsafeRenderedHtml } from "../src/rendered-html.ts";
import { parseJsonText, readJsonFileSync } from "./json-file.mjs";
import { sameMarkdownParagraph } from "./reference-markdown.mjs";
import { assertSafeRenderedHtmlEntries } from "./rendered-html.mjs";
import { highlightScaffoldScheme } from "./source-highlight.mjs";
import { documentedSourceUnit, sourceSnippet } from "./source-snippets.mjs";

const siteDir = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const referencePath = resolve(siteDir, "public/reference.static.json");
const reference = parseReferenceDocument(
  readJsonFileSync(referencePath, "static reference"),
);
const searchIndex = createReferenceSearchIndex(reference.entries);
const requiredCapabilityArrayFields = ["library_name", "bridge_library_name"];
const requiredCapabilityStringFields = [
  "library",
  "bridge_library",
  "effect",
  "docs_source",
  "notes",
];

assertNoNamedEntries(
  reference.entries.filter((entry) => entry.hidden),
  "Static reference must not include hidden entries",
);

assertNoNamedCapabilities(
  reference.capabilities.filter(
    (capability) => !hasCompleteCapabilityMetadata(capability),
  ),
  "Expected complete capability metadata",
);

assert(targetIdFromHash("") === null, "Empty hashes should not target an id");
assert(
  targetIdFromHash("#entry-tool%2Fpath") === "entry-tool/path",
  "Valid hash fragments should decode",
);
assert(
  targetIdFromHash("#%E0%A4%A") === "%E0%A4%A",
  "Invalid hash fragments should not throw",
);
assert(
  !containsUnsafeRenderedHtml('<p><code>(tool "ok")</code></p>') &&
    containsUnsafeRenderedHtml('<a href="javascript:alert(1)">bad</a>'),
  "Rendered HTML policy should allow normal markup and reject unsafe links",
);
assert(
  entryId("tool/path") !== entryId("tool-path") &&
    entryId("command/path") !== entryId("command/path?"),
  "Entry ids should disambiguate symbols that share a readable anchor",
);
assert(
  groupId("Reference") === "group-reference" &&
    groupId("Reference") !== "reference",
  "Group ids should stay in a namespace that cannot collide with page sections",
);
assert(
  parseReferenceDocument(referenceDocumentFixture()).entries[0].name ===
    "fixture",
  "Reference document parser should accept complete static payloads",
);
assertThrows(
  () =>
    parseReferenceDocument({
      title: "Broken",
      capabilities: [],
      entries: [{ name: "broken" }],
    }),
  (error) =>
    error.message.includes("entry 0 (broken)") &&
    error.message.includes("kind"),
  "Reference document parser should reject incomplete entries",
);
assertThrows(
  () =>
    parseReferenceDocument(
      referenceDocumentFixture({
        entries: [
          (() => {
            const entry = referenceEntryFixture();
            delete entry.hidden;
            return entry;
          })(),
        ],
      }),
    ),
  (error) => error.message.includes("entry 0 (fixture).hidden"),
  "Reference document parser should require explicit hidden flags",
);
assertThrows(
  () =>
    parseReferenceDocument(
      referenceDocumentFixture({
        entries: [referenceEntryFixture({ rendered_markdown: "" })],
      }),
    ),
  (error) =>
    error.message.includes("entry 0 (fixture).rendered_markdown") &&
    error.message.includes("non-empty"),
  "Reference document parser should reject empty rendered markdown",
);
assertThrows(
  () =>
    parseReferenceDocument(
      referenceDocumentFixture({
        entries: [
          referenceEntryFixture({
            rendered: { rawMarkdownHtml: null },
          }),
        ],
      }),
    ),
  (error) => error.message.includes("rendered data.params"),
  "Reference document parser should reject incomplete rendered entry data",
);
assertThrows(
  () =>
    parseReferenceDocument(
      referenceDocumentFixture({
        capabilities: [
          {
            ...referenceCapabilityFixture(),
            modes: { catalog: true },
          },
        ],
      }),
    ),
  (error) => error.message.includes("capability 0 modes"),
  "Reference document parser should reject malformed capabilities",
);
assertThrows(
  () =>
    parseReferenceDocument(
      referenceDocumentFixture({
        entries: [
          referenceEntryFixture({ name: "duplicate" }),
          referenceEntryFixture({ name: "duplicate" }),
        ],
      }),
    ),
  (error) =>
    error.message.includes("duplicate reference entry name") &&
    error.message.includes("duplicate"),
  "Reference document parser should reject duplicate entry names",
);
assertThrows(
  () =>
    parseReferenceDocument(
      referenceDocumentFixture({
        entries: [
          referenceEntryFixture({ name: "build-tool", group: "Build tools" }),
          referenceEntryFixture({ name: "build-path", group: "Build/tools" }),
        ],
      }),
    ),
  (error) =>
    error.message.includes("duplicate reference group id") &&
    error.message.includes("build-tools"),
  "Reference document parser should reject colliding group ids",
);
assertThrows(
  () =>
    parseReferenceDocument(
      referenceDocumentFixture({
        entries: [referenceEntryFixture({ name: "   " })],
      }),
    ),
  (error) =>
    error.message.includes("entry 0.name") &&
    error.message.includes("non-empty"),
  "Reference document parser should reject empty entry names",
);
assertThrows(
  () =>
    parseReferenceDocument(
      referenceDocumentFixture({
        entries: [
          referenceEntryFixture({ params: [{ name: " ", summary: "Input." }] }),
        ],
      }),
    ),
  (error) =>
    error.message.includes("entry 0 (fixture) param 0.name") &&
    error.message.includes("non-empty"),
  "Reference document parser should reject empty parameter names",
);
assertThrows(
  () =>
    parseReferenceDocument(
      referenceDocumentFixture({
        entries: [referenceEntryFixture({ see: [" "] })],
      }),
    ),
  (error) =>
    error.message.includes("entry 0 (fixture).see") &&
    error.message.includes("non-empty strings"),
  "Reference document parser should reject empty string array items",
);
assertThrows(
  () =>
    parseReferenceDocument(
      referenceDocumentFixture({
        entries: [
          referenceEntryFixture({
            rendered: {
              ...renderedEntryFixture(),
              params: [{ name: " ", summaryHtml: "<p>Input.</p>" }],
            },
          }),
        ],
      }),
    ),
  (error) =>
    error.message.includes("rendered param 0.name") &&
    error.message.includes("non-empty"),
  "Reference document parser should reject empty rendered parameter names",
);
assertThrows(
  () =>
    parseReferenceDocument(
      referenceDocumentFixture({
        entries: [referenceEntryFixture({ name: "???" })],
      }),
    ),
  (error) =>
    error.message.includes("empty reference entry anchor") &&
    error.message.includes("???"),
  "Reference document parser should reject entry names without readable anchors",
);
assertThrows(
  () =>
    parseReferenceDocument(
      referenceDocumentFixture({
        entries: [referenceEntryFixture({ group: "..." })],
      }),
    ),
  (error) =>
    error.message.includes("empty reference group id") &&
    error.message.includes("..."),
  "Reference document parser should reject groups without readable anchors",
);
assertThrows(
  () =>
    parseReferenceDocument(
      referenceDocumentFixture({
        capabilities: [
          referenceCapabilityFixture({ library: "duplicate" }),
          referenceCapabilityFixture({ library: "duplicate" }),
        ],
      }),
    ),
  (error) =>
    error.message.includes("duplicate capability library") &&
    error.message.includes("duplicate"),
  "Reference document parser should reject duplicate capability libraries",
);
assertThrows(
  () =>
    parseReferenceDocument(
      referenceDocumentFixture({
        entries: [
          referenceEntryFixture({
            rendered: {
              ...renderedEntryFixture(),
              rawMarkdownHtml: '<img src="x" onerror="alert(1)">',
            },
          }),
        ],
      }),
    ),
  (error) =>
    error.message.includes("unsafe rendered HTML") &&
    error.message.includes("rawMarkdownHtml"),
  "Reference document parser should reject unsafe raw markdown HTML",
);
assertThrows(
  () =>
    parseReferenceDocument(
      referenceDocumentFixture({
        entries: [
          referenceEntryFixture({
            rendered: {
              ...renderedEntryFixture(),
              params: [
                {
                  name: "input",
                  summaryHtml: '<a href="javascript:alert(1)">bad</a>',
                },
              ],
            },
          }),
        ],
      }),
    ),
  (error) =>
    error.message.includes("unsafe rendered HTML") &&
    error.message.includes("summaryHtml"),
  "Reference document parser should reject unsafe rendered param HTML",
);
assertThrows(
  () =>
    parseReferenceDocument(
      referenceDocumentFixture({
        entries: [
          referenceEntryFixture({
            rendered: {
              ...renderedEntryFixture(),
              returnsHtml: "<script>alert(1)</script>",
            },
          }),
        ],
      }),
    ),
  (error) =>
    error.message.includes("unsafe rendered HTML") &&
    error.message.includes("returnsHtml"),
  "Reference document parser should reject unsafe rendered return HTML",
);
assertThrows(
  () =>
    parseReferenceDocument(
      referenceDocumentFixture({
        entries: [
          referenceEntryFixture({
            rendered: {
              ...renderedEntryFixture(),
              sourceSnippet: {
                label: "source.scm:1",
                code: "(display 1)",
                html: "<pre><code><style>bad</style></code></pre>",
                startLine: 1,
              },
            },
          }),
        ],
      }),
    ),
  (error) =>
    error.message.includes("unsafe rendered HTML") &&
    error.message.includes("source snippet.html"),
  "Reference document parser should reject unsafe source snippet HTML",
);

const expectations = [
  {
    query: "tool path",
    expected: "tool/path",
  },
  {
    query: "ctlg tool",
    expected: "catalog/tool",
  },
  {
    query: "tool",
    expected: "tool",
  },
  {
    query: "src dsl std catalog tool scm",
    expected: "catalog/tool",
  },
  {
    query: "context read only",
    expected: "source/dir",
  },
  {
    query: "ripgrep",
    expected: "catalog/tool",
  },
];

for (const { query, expected } of expectations) {
  expectFirstSearchResult(searchIndex, query, expected);
}

assert(
  searchIndex.search("nope", 5).length === 0,
  "Unrelated searches should not return typo-distance prose matches",
);

assert(
  searchIndex.suggest("catalgo", 5)[0]?.entry.name === "catalog",
  "Search suggestions should recover close symbol transpositions",
);
assert(
  searchIndex.suggest("sourcpath", 5)[0]?.entry.name === "source/path",
  "Search suggestions should recover close path-like symbols",
);
assert(
  searchIndex.suggest("catlgtool", 5)[0]?.entry.name === "catalog/tool",
  "Search suggestions should recover compact compound symbols",
);
assert(
  searchIndex.suggest("zzzzzzz", 5).length === 0 &&
    searchIndex.suggest("no-such-query", 5).length === 0,
  "Search suggestions should not return unrelated noise",
);

assert(
  searchIndex
    .search("ctlg", 10)
    .every((result) => result.entry.group === "Catalog"),
  "Short fuzzy group searches should stay within the matching group",
);

assert(
  !searchIndex.search("doc field", 20).some((result) => result.entry.hidden),
  "Search results must not include hidden reference entries",
);

assert(
  searchIndex.search("t", 5).length > 0,
  "Single-character searches should return ranked results",
);

const limitProbeIndex = createReferenceSearchIndex(
  Array.from({ length: 150 }, (_, index) =>
    referenceEntryFixture({
      name: `limit-probe-${index}`,
      summary: "limit probe",
    }),
  ),
);

assert(
  limitProbeIndex.search("limit probe", 999).length === 100,
  "Search results must be capped at 100 entries",
);

for (const limit of [0, -1]) {
  assert(
    limitProbeIndex.search("limit probe", limit).length === 1,
    `Search limit ${limit} should clamp to one result`,
  );
}

assert(
  limitProbeIndex.search("limit probe", Number.NaN).length === 30,
  "NaN search limit should fall back to the default",
);

assertNoNamedEntries(
  reference.entries.filter((entry) =>
    Object.hasOwn(entry.rendered ?? {}, "markdownHtml"),
  ),
  "Static reference must not include stale rendered.markdownHtml",
);

assertNoNamedEntries(
  reference.entries.filter(
    (entry) => !Object.hasOwn(entry.rendered ?? {}, "rawMarkdownHtml"),
  ),
  "Expected rendered.rawMarkdownHtml for all entries",
);

assertNoNamedEntries(
  reference.entries.filter(
    (entry) => hasHtml(entry.rendered?.rawMarkdownHtml) && !entry.raw_markdown,
  ),
  "Raw markdown HTML must come from raw_markdown",
);

assertNoNamedEntries(
  reference.entries.filter(
    (entry) =>
      hasHtml(entry.rendered?.rawMarkdownHtml) &&
      sameMarkdownParagraph(entry.summary, entry.raw_markdown),
  ),
  "Raw markdown HTML should omit docs that repeat the summary",
);

assertNoNamedEntries(
  reference.entries.filter(
    (entry) =>
      typeof entry.rendered_markdown !== "string" ||
      entry.rendered_markdown.trim().length === 0,
  ),
  "Expected rendered_markdown for all entries",
);

assertNoNamedEntries(
  reference.entries.filter((entry) =>
    hasExcessiveGeneratedMarkdownBreak(entry.rendered_markdown),
  ),
  "Rendered markdown must not contain excessive blank lines before generated sections",
);

assertNoNamedEntries(
  reference.entries.filter(
    (entry) =>
      !Object.hasOwn(entry, "raw_markdown") ||
      entry.raw_markdown !== entry.markdown,
  ),
  "Expected raw_markdown to mirror markdown for all entries",
);

assertSafeRenderedHtmlEntries(reference.entries);

const metadataSearchIndex = createReferenceSearchIndex([
  ...reference.entries,
  referenceEntryFixture({
    name: "metadata-probe",
    stability: "experimental",
    since: "9.9.9",
  }),
]);
for (const query of ["experimental", "9.9.9"]) {
  const matches = metadataSearchIndex.search(query, 10);
  assert(
    matches.some((result) => result.entry.name === "metadata-probe"),
    `Expected lifecycle metadata search for "${query}"`,
  );
}

const entriesWithSource = reference.entries.filter(
  (entry) => entry.source && entry.range,
);
assertNoNamedEntries(
  entriesWithSource.filter(
    (entry) => !entry.source_location?.startsWith(`${entry.source}:`),
  ),
  "Expected source_location for all source-backed entries",
);

assertNoNamedEntries(
  entriesWithSource.filter(
    (entry) =>
      !entry.rendered?.sourceSnippet?.label ||
      !entry.rendered.sourceSnippet.html.includes("<pre"),
  ),
  "Expected source snippets for all source-backed entries",
);

const entriesWithMismatchedSnippetLabels = entriesWithSource.filter((entry) => {
  const label = entry.rendered?.sourceSnippet?.label;
  const sourceLocation = entry.source_location;
  if (!label || !sourceLocation) {
    return true;
  }
  const expectedLine = sourceLocation.split(":").at(-1);
  return !expectedLine || !label.endsWith(`:${expectedLine}`);
});

assertNoNamedEntries(
  entriesWithMismatchedSnippetLabels,
  "Expected source snippet labels to match source_location lines",
);

const parserProbe = [
  '(doc-next #:summary "keeps (parens) in docs")',
  "; comments between docs and definitions are ignored",
  "(define (parser-probe input)",
  "  ; ignored ) in comment",
  '  (display "ignored ) in string")',
  "  input)",
].join("\n");
const parserProbeOffset = parserProbe.indexOf("parser-probe");
const parserProbeUnit = documentedSourceUnit(parserProbe, parserProbeOffset);
assert(
  parserProbeUnit?.code.startsWith("(doc-next"),
  "Source parser should include doc-next before definitions",
);
assert(
  parserProbeUnit?.code.includes('"ignored ) in string"'),
  "Source parser should preserve strings with parentheses",
);

const snippetProbe = sourceSnippet(
  {
    source: "src/dsl/std/catalog/tool.scm",
    range: {
      line: 2,
      start: parserProbe.split("\n")[2].indexOf("parser-probe"),
      length: "parser-probe".length,
    },
  },
  { "src/dsl/std/catalog/tool.scm": parserProbe },
);
assert(
  snippetProbe?.label === "catalog/tool.scm:3" &&
    snippetProbe.startLine === 1 &&
    snippetProbe.code.includes("parser-probe"),
  "Source snippet labels and code should follow entry ranges",
);
assert(
  sourceSnippet(
    {
      source: "src/dsl/std/catalog/tool.scm",
      range: {
        line: 2,
        start: -1,
        length: "parser-probe".length,
      },
    },
    { "src/dsl/std/catalog/tool.scm": parserProbe },
  ) === null,
  "Source snippets should ignore malformed ranges",
);
assert(
  sourceSnippet(
    {
      source: "src/dsl/std/catalog/tool.scm",
      range: {
        line: 2,
        start: 999,
        length: "parser-probe".length,
      },
    },
    { "src/dsl/std/catalog/tool.scm": parserProbe },
  ) === null,
  "Source snippets should ignore out-of-bounds source positions",
);
assert(
  sourceSnippet(
    {
      source: "src/dsl/std/catalog/tool.scm",
      range: {
        line: parserProbe.split("\n").length,
        start: 0,
        length: "parser-probe".length,
      },
    },
    { "src/dsl/std/catalog/tool.scm": parserProbe },
  ) === null,
  "Source snippets should ignore source lines outside the file",
);
assert(
  sourceSnippet(
    {
      source: "src/dsl/std/catalog/tool.scm",
      range: {
        line: 2,
        start: parserProbe.split("\n")[2].indexOf("parser-probe"),
        length: "parser-probe".length,
      },
    },
    Object.create({ "src/dsl/std/catalog/tool.scm": parserProbe }),
  ) === null,
  "Source snippets should ignore inherited source bundle properties",
);

const highlightedProbe = highlightScaffoldScheme(
  '(display "<unsafe>")',
  JSON.stringify([
    {
      line: 0,
      start: 1,
      length: 7,
      token_type: "function",
      modifiers: ["defaultLibrary"],
    },
    {
      line: 0,
      start: 999,
      length: 4,
      token_type: "ignored",
      modifiers: [],
    },
    {
      line: -1,
      start: 0,
      length: 1,
      token_type: "ignored",
      modifiers: [],
    },
    {
      line: 0,
      start: 10,
      length: 8,
      token_type: "string",
      modifiers: ["deprecated", "unknown"],
    },
  ]),
);
assert(
  highlightedProbe.includes('class="tok tok-function tok-defaultLibrary"') &&
    highlightedProbe.includes('class="tok tok-string tok-deprecated"') &&
    highlightedProbe.includes("&lt;unsafe&gt;") &&
    !highlightedProbe.includes("tok-ignored") &&
    !highlightedProbe.includes("tok-unknown"),
  "Source highlighting should escape code and ignore invalid token spans and modifiers",
);
assert(
  !highlightScaffoldScheme(
    "(display 1)",
    JSON.stringify([
      {
        line: 0,
        start: 1,
        length: 7,
        token_type: "not-a-token",
        modifiers: ["deprecated"],
      },
    ]),
  ).includes('class="tok'),
  "Source highlighting should ignore unknown token types",
);
assert(
  highlightScaffoldScheme('(display "<fallback>")', "not json").includes(
    "&lt;fallback&gt;",
  ),
  "Source highlighting should fall back to escaped code for malformed tokens",
);
assert(
  !highlightScaffoldScheme("(display 1)", "{}").includes('class="tok'),
  "Source highlighting should ignore non-array token payloads",
);

assert(
  parseJsonText('{"ok":true}', "json fixture").ok === true,
  "JSON helper should parse valid JSON",
);
assertThrows(
  () => parseJsonText("{", "broken json fixture"),
  (error) =>
    error.message.includes("broken json fixture") &&
    error.message.includes("JSON"),
  "JSON helper should include file context in parse failures",
);

console.log("Reference checks passed");

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function assertThrows(fn, predicate, message) {
  try {
    fn();
  } catch (error) {
    assert(
      error instanceof Error && predicate(error),
      `${message}, got ${errorMessage(error)}`,
    );
    return;
  }

  throw new Error(`${message}, but no error was thrown`);
}

function assertNoNamedEntries(entries, message) {
  if (entries.length > 0) {
    throw new Error(`${message}, found ${entryNames(entries)}`);
  }
}

function assertNoNamedCapabilities(capabilities, message) {
  if (capabilities.length > 0) {
    throw new Error(`${message}, found ${capabilityNames(capabilities)}`);
  }
}

function expectFirstSearchResult(index, query, expected) {
  const [first] = index.search(query, 5);

  assert(
    first?.entry.name === expected,
    `Expected "${query}" to rank "${expected}" first, got ${first?.entry.name ?? "no result"}`,
  );
}

function entryNames(entries) {
  return entries
    .slice(0, 5)
    .map((entry) => entry.name)
    .join(", ");
}

function capabilityNames(capabilities) {
  return capabilities
    .slice(0, 5)
    .map((capability) => capability.library ?? "<missing library>")
    .join(", ");
}

function hasCompleteCapabilityMetadata(capability) {
  return (
    requiredCapabilityArrayFields.every((field) =>
      nonEmptyArray(capability[field]),
    ) &&
    requiredCapabilityStringFields.every((field) => hasText(capability[field]))
  );
}

function nonEmptyArray(value) {
  return Array.isArray(value) && value.length > 0;
}

function hasText(value) {
  return typeof value === "string" && value.length > 0;
}

function hasExcessiveGeneratedMarkdownBreak(markdown) {
  return [
    "\n\n\n**Parameters**",
    "\n\n\n**Returns:**",
    "\n\n\n**Example**",
    "\n\n\n**Deprecated:**",
    "\n\n\n**See also:**",
  ].some((section) => markdown.includes(section));
}

function referenceEntryFixture(overrides) {
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
    rendered: renderedEntryFixture(),
    ...overrides,
  };
}

function renderedEntryFixture(overrides = {}) {
  return {
    rawMarkdownHtml: null,
    params: [],
    returnsHtml: null,
    sourceSnippet: null,
    ...overrides,
  };
}

function referenceDocumentFixture(overrides) {
  return {
    title: "Reference fixture",
    capabilities: [referenceCapabilityFixture()],
    catalog_schema: {},
    entries: [referenceEntryFixture({})],
    ...overrides,
  };
}

function referenceCapabilityFixture(overrides = {}) {
  return {
    library_name: ["catalog"],
    library: "catalog",
    bridge_library_name: ["catalog-bridge"],
    bridge_library: "catalog-bridge",
    effect: "filesystem",
    modes: {
      catalog: "available",
      test: "available",
      editor: "unavailable",
      wasm: "unavailable",
    },
    docs_source: "fixture",
    notes: "Fixture capability.",
    ...overrides,
  };
}

function hasHtml(value) {
  return typeof value === "string" && value.trim().length > 0;
}

function errorMessage(error) {
  return error instanceof Error ? error.message : String(error);
}
