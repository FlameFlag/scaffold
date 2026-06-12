import initScaffoldWasm, {
  completionItemsScaffoldSchemeForDocument,
  diagnoseScaffoldScheme,
  formatScaffoldScheme,
  referenceCapabilitiesScaffoldScheme,
  referenceCatalogSchemaScaffoldScheme,
  referenceEntriesScaffoldSchemeForWorkspace,
  semanticTokensScaffoldScheme,
} from "../../editors/vscode/wasm/scaffold_wasm.js";
import wasmUrl from "../../editors/vscode/wasm/scaffold_wasm_bg.wasm?url";
import sourceWorkspace from "../../editors/vscode/scaffold-sources.json";
import type { ReferenceCapability, ReferenceDocument, ReferenceEntry } from "./reference";

type SourceWorkspace = Record<string, string>;

type SemanticToken = {
  text: string;
  line: number;
  start: number;
  length: number;
  token_type: "function" | "keyword" | "string" | "comment" | "parameter";
  modifiers: string[];
};

export type WasmCompletionItem = {
  label: string;
  kind: "function" | "keyword";
  detail?: string;
  documentation: string;
  insert_text: string;
  insert_text_is_snippet: boolean;
  sort_text: string;
  deprecated: boolean;
};

export type WasmDiagnostic = {
  message: string;
  offset: number;
  length: number;
  severity: "error" | "warning";
  code: string;
  data?: {
    name: string;
    line: number;
  };
};

const sources = sourceWorkspace as SourceWorkspace;
const workspaceJson = JSON.stringify(sources);

let wasmReady: Promise<unknown> | undefined;

function ensureWasm() {
  wasmReady ??= initScaffoldWasm({ module_or_path: wasmUrl });
  return wasmReady;
}

function parseJson<T>(value: string): T {
  return JSON.parse(value) as T;
}

export async function loadReferenceDocument(): Promise<ReferenceDocument> {
  await ensureWasm();

  return {
    title: "Scaffold Scheme Reference",
    capabilities: parseJson<ReferenceCapability[]>(
      referenceCapabilitiesScaffoldScheme(),
    ),
    catalog_schema: parseJson<unknown>(referenceCatalogSchemaScaffoldScheme()),
    entries: parseJson<ReferenceEntry[]>(
      referenceEntriesScaffoldSchemeForWorkspace(workspaceJson),
    ).filter((entry) => !entry.hidden),
  };
}

export async function completionItemsForDocument(
  code: string,
): Promise<WasmCompletionItem[]> {
  await ensureWasm();
  return parseJson<WasmCompletionItem[]>(
    completionItemsScaffoldSchemeForDocument(code, workspaceJson),
  );
}

export async function diagnoseDocument(code: string): Promise<WasmDiagnostic[]> {
  await ensureWasm();
  return parseJson<WasmDiagnostic[]>(diagnoseScaffoldScheme(code));
}

export async function formatDocument(code: string): Promise<string> {
  await ensureWasm();
  return formatScaffoldScheme(code);
}

export type SourceSnippet = {
  label: string;
  code: string;
  startLine: number;
};

export function sourceSnippet(entry: ReferenceEntry): SourceSnippet | null {
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

export async function highlightScaffoldScheme(code: string) {
  await ensureWasm();
  const tokens = parseJson<SemanticToken[]>(semanticTokensScaffoldScheme(code));
  const tokensByLine = new Map<number, SemanticToken[]>();
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

function shortSourcePath(sourcePath: string) {
  return sourcePath.replace(/^src\/dsl\/std\//, "").replace(/^src\//, "");
}

function offsetForPosition(lines: string[], zeroBasedLine: number, character: number) {
  let offset = 0;

  for (let index = 0; index < zeroBasedLine; index += 1) {
    offset += (lines[index]?.length ?? 0) + 1;
  }

  return offset + character;
}

function documentedSourceUnit(source: string, offset: number) {
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

function previousForm(source: string, beforeOffset: number) {
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

function formStartEndingAt(source: string, targetEnd: number) {
  const stack: number[] = [];
  let inString = false;
  let escaped = false;
  let inComment = false;

  for (let index = 0; index <= targetEnd; index += 1) {
    const character = source[index];

    if (inComment) {
      inComment = character !== "\n";
      continue;
    }

    if (inString) {
      if (escaped) {
        escaped = false;
      } else if (character === "\\") {
        escaped = true;
      } else if (character === "\"") {
        inString = false;
      }
      continue;
    }

    if (character === ";") {
      inComment = true;
    } else if (character === "\"") {
      inString = true;
    } else if (character === "(") {
      stack.push(index);
    } else if (character === ")") {
      const start = stack.pop();

      if (index === targetEnd) {
        return start ?? null;
      }
    }
  }

  return null;
}

function parenStackAt(source: string, offset: number) {
  const stack: number[] = [];
  let inString = false;
  let escaped = false;
  let inComment = false;

  for (let index = 0; index < Math.min(offset, source.length); index += 1) {
    const character = source[index];

    if (inComment) {
      inComment = character !== "\n";
      continue;
    }

    if (inString) {
      if (escaped) {
        escaped = false;
      } else if (character === "\\") {
        escaped = true;
      } else if (character === "\"") {
        inString = false;
      }
      continue;
    }

    if (character === ";") {
      inComment = true;
    } else if (character === "\"") {
      inString = true;
    } else if (character === "(") {
      stack.push(index);
    } else if (character === ")") {
      stack.pop();
    }
  }

  return stack;
}

function matchingParen(source: string, start: number) {
  let depth = 0;
  let inString = false;
  let escaped = false;
  let inComment = false;

  for (let index = start; index < source.length; index += 1) {
    const character = source[index];

    if (inComment) {
      inComment = character !== "\n";
      continue;
    }

    if (inString) {
      if (escaped) {
        escaped = false;
      } else if (character === "\\") {
        escaped = true;
      } else if (character === "\"") {
        inString = false;
      }
      continue;
    }

    if (character === ";") {
      inComment = true;
    } else if (character === "\"") {
      inString = true;
    } else if (character === "(") {
      depth += 1;
    } else if (character === ")") {
      depth -= 1;

      if (depth === 0) {
        return index;
      }
    }
  }

  return null;
}

function formHead(source: string, start: number) {
  const match = source.slice(start + 1).match(/^\s*([^\s()]+)/);
  return match?.[1] ?? null;
}

function trimIndent(value: string) {
  const lines = value.split("\n");
  const indent = Math.min(
    ...lines
      .filter((line) => line.trim().length > 0)
      .map((line) => line.match(/^\s*/)?.[0].length ?? 0),
  );

  return lines.map((line) => line.slice(indent)).join("\n");
}

function highlightLine(line: string, tokens: SemanticToken[]) {
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

function tokenClass(token: SemanticToken) {
  return [
    "tok",
    `tok-${token.token_type}`,
    ...token.modifiers.map((modifier) => `tok-${modifier}`),
  ].join(" ");
}

function escapeHtml(value: string) {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;");
}
