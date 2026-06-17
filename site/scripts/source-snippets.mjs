export function sourceSnippet(entry, sources) {
  const source = sourceForEntry(entry, sources);
  const range = sourceRangeForEntry(entry);
  if (!source || !range) {
    return null;
  }

  const lines = source.split("\n");
  if (!isPositionInSource(lines, range.line, range.start)) {
    return null;
  }

  const sourceLine = Math.max(1, range.line + 1);
  const offset = offsetForPosition(lines, range.line, range.start);
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

function sourceForEntry(entry, sources) {
  if (!entry.source || !Object.hasOwn(sources, entry.source)) {
    return null;
  }

  const source = sources[entry.source];
  return typeof source === "string" && source.length > 0 ? source : null;
}

function sourceRangeForEntry(entry) {
  return isSourceRange(entry.range) ? entry.range : null;
}

export function documentedSourceUnit(source, offset) {
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

function shortSourcePath(sourcePath) {
  return sourcePath.replace(/^src\/dsl\/std\//, "").replace(/^src\//, "");
}

function isSourceRange(value) {
  return (
    isObject(value) &&
    hasNonNegativeInteger(value.line) &&
    hasNonNegativeInteger(value.start) &&
    hasPositiveInteger(value.length)
  );
}

function isObject(value) {
  return typeof value === "object" && value !== null;
}

function hasNonNegativeInteger(value) {
  return Number.isInteger(value) && value >= 0;
}

function hasPositiveInteger(value) {
  return Number.isInteger(value) && value > 0;
}

function isPositionInSource(lines, zeroBasedLine, character) {
  const line = lines[zeroBasedLine];
  return line !== undefined && character < line.length;
}

function offsetForPosition(lines, zeroBasedLine, character) {
  let offset = 0;

  for (let index = 0; index < zeroBasedLine; index += 1) {
    offset += (lines[index]?.length ?? 0) + 1;
  }

  return offset + character;
}

function previousForm(source, beforeOffset) {
  const parentDepth = parenStackAt(source, beforeOffset).length;
  let previous = null;
  const stack = [];

  scanSchemeForms(source, beforeOffset - 1, (event) => {
    if (event.kind === "open") {
      stack.push(event.index);
      return;
    }

    const start = stack.pop();
    if (start !== undefined && stack.length === parentDepth) {
      previous = { start, end: event.index };
    }
  });

  return previous;
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
    return commentState(character);
  }

  if (state === "string") {
    return stringState(character, escaped);
  }

  return codeState(character, index);
}

function commentState(character) {
  return {
    state: character === "\n" ? "code" : "comment",
    escaped: false,
    event: null,
  };
}

function stringState(character, escaped) {
  if (escaped) {
    return { state: "string", escaped: false, event: null };
  }

  if (character === "\\") {
    return { state: "string", escaped: true, event: null };
  }

  return {
    state: character === '"' ? "code" : "string",
    escaped: false,
    event: null,
  };
}

function codeState(character, index) {
  if (character === "(") {
    return { state: "code", escaped: false, event: { kind: "open", index } };
  }

  if (character === ")") {
    return { state: "code", escaped: false, event: { kind: "close", index } };
  }

  if (character === ";") {
    return { state: "comment", escaped: false, event: null };
  }

  if (character === '"') {
    return { state: "string", escaped: false, event: null };
  }

  return { state: "code", escaped: false, event: null };
}

function parenStackAt(source, offset) {
  const stack = [];

  scanSchemeForms(source, offset - 1, (event) => {
    if (event.kind === "open") {
      stack.push(event.index);
    } else if (stack.length > 0) {
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
