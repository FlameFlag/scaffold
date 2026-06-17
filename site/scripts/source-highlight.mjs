const semanticTokenTypes = new Set([
  "function",
  "keyword",
  "string",
  "comment",
  "parameter",
]);
const semanticTokenModifiers = new Set([
  "defaultLibrary",
  "documentation",
  "deprecated",
  "definition",
]);

export function highlightScaffoldScheme(code, semanticTokensJson) {
  const tokens = parseSemanticTokens(semanticTokensJson);
  const tokensByLine = tokensByLineNumber(tokens);

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

function parseSemanticTokens(value) {
  try {
    const tokens = JSON.parse(value);
    return Array.isArray(tokens)
      ? tokens.map(normalizeSemanticToken).filter((token) => token !== null)
      : [];
  } catch {
    return [];
  }
}

function normalizeSemanticToken(token) {
  if (
    !isObject(token) ||
    !hasNonNegativeInteger(token.line) ||
    !hasNonNegativeInteger(token.start) ||
    !hasPositiveInteger(token.length) ||
    !isSemanticTokenType(token.token_type)
  ) {
    return null;
  }

  return {
    line: token.line,
    start: token.start,
    length: token.length,
    tokenType: token.token_type,
    modifiers: semanticTokenModifiersFor(token.modifiers),
  };
}

function isObject(value) {
  return typeof value === "object" && value !== null;
}

function isSemanticTokenType(value) {
  return typeof value === "string" && semanticTokenTypes.has(value);
}

function semanticTokenModifiersFor(value) {
  return Array.isArray(value)
    ? value.filter(
        (modifier) =>
          typeof modifier === "string" && semanticTokenModifiers.has(modifier),
      )
    : [];
}

function highlightLine(line, tokens) {
  let cursor = 0;
  let html = "";

  for (const token of tokens) {
    const span = tokenSpan(line, token);
    if (!span || span.start < cursor) {
      continue;
    }

    html += escapeHtml(line.slice(cursor, span.start));
    html += `<span class="${tokenClass(token)}">${escapeHtml(line.slice(span.start, span.end))}</span>`;
    cursor = span.end;
  }

  return html + escapeHtml(line.slice(cursor));
}

function tokenSpan(line, token) {
  if (!hasValidTokenRange(line, token)) {
    return null;
  }

  const end = Math.min(line.length, token.start + token.length);
  return end > token.start ? { start: token.start, end } : null;
}

function hasValidTokenRange(line, token) {
  return token.start < line.length;
}

function hasNonNegativeInteger(value) {
  return Number.isInteger(value) && value >= 0;
}

function hasPositiveInteger(value) {
  return Number.isInteger(value) && value > 0;
}

function tokenClass(token) {
  return [
    "tok",
    `tok-${token.tokenType}`,
    ...token.modifiers.map((modifier) => `tok-${modifier}`),
  ].join(" ");
}

function tokensByLineNumber(tokens) {
  const tokensByLine = new Map();

  for (const token of tokens) {
    const lineTokens = tokensByLine.get(token.line) ?? [];
    lineTokens.push(token);
    tokensByLine.set(token.line, lineTokens);
  }

  return tokensByLine;
}

function escapeHtml(value) {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;");
}
