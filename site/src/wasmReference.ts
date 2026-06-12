import initScaffoldWasm, {
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

type ScanEvent =
  | { kind: "open"; index: number }
  | { kind: "close"; index: number };
type ScanState = "code" | "string" | "comment";
type ScanResult = {
  state: ScanState;
  escaped: boolean;
  event: ScanEvent | null;
};

function stringState(character: string, escaped: boolean) {
  if (escaped) {
    return { state: "string" as ScanState, escaped: false, event: null };
  }

  if (character === "\\") {
    return { state: "string" as ScanState, escaped: true, event: null };
  }

  return {
    state: character === "\"" ? "code" : ("string" as ScanState),
    escaped: false,
    event: null,
  };
}

function commentState(character: string) {
  return {
    state: character === "\n" ? "code" : ("comment" as ScanState),
    escaped: false,
    event: null,
  };
}

function scanEvent(character: string, index: number): ScanEvent | null {
  if (character === "(") {
    return { kind: "open", index };
  }

  return character === ")" ? { kind: "close", index } : null;
}

function codeState(character: string, index: number): ScanResult {
  if (character === ";") {
    return { state: "comment", escaped: false, event: null };
  }

  if (character === "\"") {
    return { state: "string", escaped: false, event: null };
  }

  return { state: "code", escaped: false, event: scanEvent(character, index) };
}

function scanCharacter(
  state: ScanState,
  escaped: boolean,
  character: string,
  index: number,
): ScanResult {
  if (state === "comment") {
    return commentState(character);
  }

  return state === "string"
    ? stringState(character, escaped)
    : codeState(character, index);
}

function scanSchemeForms(
  source: string,
  endOffset: number,
  visit: (event: ScanEvent) => boolean | void,
) {
  let state: ScanState = "code";
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

function formStartEndingAt(source: string, targetEnd: number) {
  const stack: number[] = [];
  let formStart: number | null = null;

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

function parenStackAt(source: string, offset: number) {
  const stack: number[] = [];

  scanSchemeForms(source, offset - 1, (event) => {
    if (event.kind === "open") {
      stack.push(event.index);
    } else {
      stack.pop();
    }
  });

  return stack;
}

function matchingParen(source: string, start: number) {
  let depth = 0;
  let match: number | null = null;

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
