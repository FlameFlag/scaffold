import { mkdir, readFile, rename, rm, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";
import { marked } from "marked";
import initScaffoldWasm, {
  semanticTokensScaffoldScheme,
} from "../../editors/vscode/wasm/scaffold_wasm.js";
import { readJsonFile } from "./json-file.mjs";
import { sameMarkdownParagraph } from "./reference-markdown.mjs";
import { assertSafeRenderedHtmlEntries } from "./rendered-html.mjs";
import { highlightScaffoldScheme } from "./source-highlight.mjs";
import { sourceSnippet } from "./source-snippets.mjs";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const siteDir = resolve(scriptDir, "..");
const repoDir = resolve(siteDir, "..");
const referencePath = resolve(siteDir, ".generated/reference.json");
const outputPath = resolve(siteDir, "public/reference.static.json");
const temporaryOutputPath = `${outputPath}.${process.pid}.${Date.now()}.tmp`;
const sourcesPath = resolve(repoDir, "editors/vscode/scaffold-sources.json");
const wasmPath = resolve(repoDir, "editors/vscode/wasm/scaffold_wasm_bg.wasm");
const checkOnly = scriptArgs().check;

marked.use({
  gfm: true,
  breaks: false,
});

const reference = await readJsonFile(referencePath, "generated reference");
const sources = await readJsonFile(sourcesPath, "bundled source map");
const wasmBytes = await Bun.file(wasmPath).arrayBuffer();

await initScaffoldWasm({ module_or_path: wasmBytes });

const renderedReference = {
  ...reference,
  entries: reference.entries
    .filter((entry) => !entry.hidden)
    .map((entry) => ({
      ...entry,
      rendered: {
        rawMarkdownHtml: rawMarkdownHtml(entry),
        params: entry.params.map((param) => ({
          name: param.name,
          summaryHtml: String(marked.parseInline(param.summary)),
        })),
        returnsHtml: entry.returns
          ? String(marked.parseInline(entry.returns))
          : null,
        sourceSnippet: renderSourceSnippet(entry),
      },
    })),
};

const renderedReferenceJson = `${JSON.stringify(renderedReference)}\n`;

assertSafeRenderedHtmlEntries(renderedReference.entries);

try {
  if (checkOnly) {
    const current = await readFile(outputPath, "utf8").catch(() => null);
    if (current !== renderedReferenceJson) {
      throw new Error(
        'public/reference.static.json is stale; run "bun run --cwd site static-reference"',
      );
    }
  } else {
    await mkdir(dirname(outputPath), { recursive: true });
    await writeFile(temporaryOutputPath, renderedReferenceJson);
    await rename(temporaryOutputPath, outputPath);

    console.log(
      `Generated ${renderedReference.entries.length} static reference entries at ${outputPath}`,
    );
  }
} finally {
  await rm(temporaryOutputPath, { force: true });
}

function rawMarkdownHtml(entry) {
  const rawMarkdown = entry.raw_markdown ?? entry.markdown;
  if (!rawMarkdown || sameMarkdownParagraph(entry.summary, rawMarkdown)) {
    return null;
  }

  return String(marked.parse(rawMarkdown));
}

function renderSourceSnippet(entry) {
  const snippet = sourceSnippet(entry, sources);

  if (!snippet) {
    return null;
  }

  return {
    ...snippet,
    html: highlightScaffoldScheme(
      snippet.code,
      semanticTokensScaffoldScheme(snippet.code),
    ),
  };
}

function scriptArgs() {
  return parseArgs({
    options: {
      check: { type: "boolean", default: false },
    },
    strict: true,
  }).values;
}
