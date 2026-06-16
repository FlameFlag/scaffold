import { readFile, rename, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import initScaffoldWasm, {
  semanticTokensScaffoldScheme,
} from "../../editors/vscode/wasm/scaffold_wasm.js";
import { marked } from "marked";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const siteDir = resolve(scriptDir, "..");
const repoDir = resolve(siteDir, "..");
const referencePath = resolve(siteDir, ".generated/reference.json");
const outputPath = resolve(siteDir, "public/reference.static.json");
const sourcesPath = resolve(repoDir, "editors/vscode/scaffold-sources.json");
const wasmPath = resolve(repoDir, "editors/vscode/wasm/scaffold_wasm_bg.wasm");

marked.use({
  gfm: true,
  breaks: false,
});

const reference = JSON.parse(await readFile(referencePath, "utf8"));
const sources = JSON.parse(await readFile(sourcesPath, "utf8"));
const wasmBytes = await Bun.file(wasmPath).arrayBuffer();

await initScaffoldWasm({ module_or_path: wasmBytes });

const renderedReference = {
  ...reference,
  entries: reference.entries
    .filter((entry) => !entry.hidden)
    .map((entry) => ({
      ...entry,
      rendered: {
        markdownHtml: entry.markdown ? String(marked.parse(entry.markdown)) : null,
        params: entry.params.map((param) => ({
          name: param.name,
          summaryHtml: String(marked.parseInline(param.summary)),
        })),
        returnsHtml: entry.returns ? String(marked.parseInline(entry.returns)) : null,
        sourceSnippet: renderSourceSnippet(entry),
      },
    })),
};

await writeFile(`${outputPath}.tmp`, `${JSON.stringify(renderedReference)}\n`);
await rename(`${outputPath}.tmp`, outputPath);

console.log(
  `Generated ${renderedReference.entries.length} static reference entries at ${outputPath}`,
);

function renderSourceSnippet(entry) {
  const snippet = sourceSnippet(entry);

  if (!snippet) {
    return null;
  }

  return {
    ...snippet,
    html: highlightScaffoldScheme(snippet.code),
  };
}

function sourceSnippet(entry) {
  if (!entry.source || !entry.range) {
    return null;
  }

  const source = sources[entry.source];
  if (!source) {
    return null;
  }

  const lines = source.split("\n");
  const sourceLine = Math.max(1, entry.range.line + 1);
  const offset = offsetForPosition(lines, entry.range.line, entry.range.start);
  const form = documentedSourceUnit(source, offset);

  if (!form) {
    return null;
  }

  return {
    label: `${shortSourcePath(entry.source)}:${sourceLine}`,
    code: form.code,
    startLine: form.startLine,
  };
}

function highlightScaffoldScheme(code) {
  const tokens = JSON.parse(semanticTokensScaffoldScheme(code));
  const tokensByLine = new Map();

  for (const token of tokens) {
    const lineTokens = tokensByLine.get(token.line) ?? [];
    lineTokens.push(token);
    tokensByLine.set(token.line, lineTokens);
  }

  return `<pre class="scaffoldCode"><code>${code
    .split("\n")
    .map((line, index) => {
      const lineTokens = (tokensByLine.get(index) ?? []).sort(
        (left, right) => left.start - right.start,
      );

      return `<span class="line">${highlightLine(line, lineTokens)}</span>`;
    })
    .join("\n")}</code></pre>`;
}

function shortSourcePath(sourcePath) {
  return sourcePath.replace(/^src\/dsl\/std\//, "").replace(/^src\//, "");
}

function offsetForPosition(lines, zeroBasedLine, character) {
  let offset = 0;

  for (let index = 0; index < zeroBasedLine; index += 1) {
    offset += (lines[index]?.length ?? 0) + 1;
  }

  return offset + character;
}

function documentedSourceUnit(source, offset) {
  const stack = parenStackAt(source, offset);
  const definitionStart =
    stack.findLast((position) => formHead(source, position) === "define") ??
    stack[0];

  if (definitionStart === undefined) {
    return null;
  }

  const definitionEnd = matchingParen(source, definitionStart);
  if (definitionEnd === null) {
    return null;
  }

  const previous = previousForm(source, definitionStart);
  const start =
    previous && formHead(source, previous.start) === "doc-next"
      ? previous.start
      : definitionStart;

  const rawCode = source.slice(start, definitionEnd + 1);
  const code = trimIndent(rawCode);
  const startLine = source.slice(0, start).split("\n").length;

  return { code, startLine };
}

function previousForm(source, beforeOffset) {
  let end = beforeOffset - 1;

  while (end >= 0 && /\s/.test(source[end])) {
    end -= 1;
  }

  if (source[end] !== ")") {
    return null;
  }

  const start = formStartEndingAt(source, end);
  return start === null ? null : { start, end };
}

function scanSchemeForms(source, endOffset, visit) {
  let state = "code";
  let escaped = false;
  const end = Math.min(endOffset, source.length - 1);

  for (let index = 0; index <= end; index += 1) {
    const character = source[index];
    const result = scanCharacter(state, escaped, character, index);

    state = result.state;
    escaped = result.escaped;

    if (result.event && visit(result.event) === false) {
      return;
    }
  }
}

function scanCharacter(state, escaped, character, index) {
  if (state === "comment") {
    return {
      state: character === "\n" ? "code" : "comment",
      escaped: false,
      event: null,
    };
  }

  if (state === "string") {
    if (escaped) {
      return { state: "string", escaped: false, event: null };
    }

    if (character === "\\") {
      return { state: "string", escaped: true, event: null };
    }

    return {
      state: character === "\"" ? "code" : "string",
      escaped: false,
      event: null,
    };
  }

  if (character === ";") {
    return { state: "comment", escaped: false, event: null };
  }

  if (character === "\"") {
    return { state: "string", escaped: false, event: null };
  }

  if (character === "(") {
    return { state: "code", escaped: false, event: { kind: "open", index } };
  }

  if (character === ")") {
    return { state: "code", escaped: false, event: { kind: "close", index } };
  }

  return { state: "code", escaped: false, event: null };
}

function formStartEndingAt(source, targetEnd) {
  const stack = [];
  let formStart = null;

  scanSchemeForms(source, targetEnd, (event) => {
    if (event.kind === "open") {
      stack.push(event.index);
      return;
    }

    const start = stack.pop();

    if (event.index === targetEnd) {
      formStart = start ?? null;
      return false;
    }
  });

  return formStart;
}

function parenStackAt(source, offset) {
  const stack = [];

  scanSchemeForms(source, offset - 1, (event) => {
    if (event.kind === "open") {
      stack.push(event.index);
    } else {
      stack.pop();
    }
  });

  return stack;
}

function matchingParen(source, start) {
  let depth = 0;
  let match = null;

  scanSchemeForms(source.slice(start), source.length - start - 1, (event) => {
    if (event.kind === "open") {
      depth += 1;
      return;
    }

    depth -= 1;

    if (depth === 0) {
      match = start + event.index;
      return false;
    }
  });

  return match;
}

function formHead(source, start) {
  const match = source.slice(start + 1).match(/^\s*([^\s()]+)/);
  return match?.[1] ?? null;
}

function trimIndent(value) {
  const lines = value.split("\n");
  const indent = Math.min(
    ...lines
      .filter((line) => line.trim().length > 0)
      .map((line) => line.match(/^\s*/)?.[0].length ?? 0),
  );

  return lines.map((line) => line.slice(indent)).join("\n");
}

function highlightLine(line, tokens) {
  let cursor = 0;
  let html = "";

  for (const token of tokens) {
    const start = token.start;
    const end = Math.min(line.length, start + token.length);

    if (start < cursor) {
      continue;
    }

    html += escapeHtml(line.slice(cursor, start));
    html += `<span class="${tokenClass(token)}">${escapeHtml(line.slice(start, end))}</span>`;
    cursor = end;
  }

  return html + escapeHtml(line.slice(cursor));
}

function tokenClass(token) {
  return [
    "tok",
    `tok-${token.token_type}`,
    ...token.modifiers.map((modifier) => `tok-${modifier}`),
  ].join(" ");
}

function escapeHtml(value) {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;");
}
