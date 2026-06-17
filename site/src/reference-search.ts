import type { ReferenceEntry } from "./reference";

type ReferenceSearchResult = {
  entry: ReferenceEntry;
  score: number;
};

type SearchField = {
  normalized: string;
  tokens: string[];
  kind: SearchFieldKind;
  weight: number;
};

type SearchFieldKind = "symbol" | "source" | "prose";

const DEFAULT_SEARCH_LIMIT = 30;
const MAX_SEARCH_LIMIT = 100;

export function createReferenceSearchIndex(entries: ReferenceEntry[]) {
  const documents = entries
    .filter((entry) => !entry.hidden)
    .map((entry) => ({
      entry,
      fields: searchFields(entry),
      suggestionCandidates: suggestionCandidates(entry),
    }));

  return {
    search(
      query: string,
      limit = DEFAULT_SEARCH_LIMIT,
    ): ReferenceSearchResult[] {
      const normalizedQuery = normalizeSearchText(query);
      const terms = searchTerms(normalizedQuery);

      if (terms.length === 0) {
        return [];
      }

      return documents
        .map(({ entry, fields }) => ({
          entry,
          score: scoreEntry(fields, terms, normalizedQuery),
        }))
        .filter((result) => result.score > 0)
        .sort(compareSearchResults)
        .slice(0, searchLimit(limit));
    },
    suggest(query: string, limit = 5): ReferenceSearchResult[] {
      const normalizedQuery = normalizeSuggestionText(query);
      const maxDistance = suggestionDistanceThreshold(normalizedQuery);

      if (maxDistance === 0) {
        return [];
      }

      return documents
        .map(({ entry, suggestionCandidates }) => ({
          entry,
          score: suggestionScore(
            normalizedQuery,
            suggestionCandidates,
            maxDistance,
          ),
        }))
        .filter((result) => result.score !== null)
        .map((result) => ({
          entry: result.entry,
          score: result.score ?? 0,
        }))
        .sort(compareSuggestionResults)
        .slice(0, searchLimit(limit));
    },
  };
}

function searchLimit(limit: number) {
  if (Number.isNaN(limit)) {
    return DEFAULT_SEARCH_LIMIT;
  }
  if (!Number.isFinite(limit)) {
    return limit > 0 ? MAX_SEARCH_LIMIT : 1;
  }
  return Math.min(MAX_SEARCH_LIMIT, Math.max(1, Math.trunc(limit)));
}

function compareSearchResults(
  left: ReferenceSearchResult,
  right: ReferenceSearchResult,
) {
  return (
    right.score - left.score || left.entry.name.localeCompare(right.entry.name)
  );
}

function compareSuggestionResults(
  left: ReferenceSearchResult,
  right: ReferenceSearchResult,
) {
  return (
    left.score - right.score || left.entry.name.localeCompare(right.entry.name)
  );
}

function searchFields(entry: ReferenceEntry): SearchField[] {
  return [
    field(entry.name, "symbol", 80),
    field(entry.group, "symbol", 28),
    field(entry.signature, "symbol", 22),
    field(entry.summary, "prose", 14),
    field(
      entry.params.map((param) => `${param.name} ${param.summary}`).join(" "),
      "prose",
      10,
    ),
    field(entry.returns, "prose", 8),
    field(entry.see.join(" "), "symbol", 8),
    field(
      `${entry.effect ?? ""} ${entry.requires_capability.join(" ")}`,
      "symbol",
      7,
    ),
    field(`${entry.since ?? ""} ${entry.stability ?? ""}`, "symbol", 7),
    field(`${entry.source ?? ""} ${entry.source_location ?? ""}`, "source", 6),
    field(entry.example, "prose", 5),
    field(entry.markdown, "prose", 5),
  ].filter((item) => item.normalized.length > 0);
}

function suggestionCandidates(entry: ReferenceEntry): SearchField[] {
  return [
    field(entry.name, "symbol", 0),
    ...entry.name
      .split(/[^a-zA-Z0-9]+/)
      .filter(Boolean)
      .map((token) => field(token, "symbol", 1)),
  ];
}

function suggestionScore(
  normalizedQuery: string,
  candidates: SearchField[],
  maxDistance: number,
) {
  const scores = candidates
    .map((candidate) => {
      const distance = editDistance(
        normalizedQuery,
        normalizeSuggestionText(candidate.normalized),
        maxDistance,
      );
      return distance <= maxDistance ? distance * 10 + candidate.weight : null;
    })
    .filter((score) => score !== null);

  return scores.length > 0 ? Math.min(...scores) : null;
}

function field(
  text: string | null | undefined,
  kind: SearchFieldKind,
  weight: number,
): SearchField {
  const normalized = normalizeSearchText(text ?? "");

  return {
    normalized,
    tokens: normalized.split(" ").filter(Boolean),
    kind,
    weight,
  };
}

function scoreEntry(
  fields: SearchField[],
  terms: string[],
  normalizedQuery: string,
) {
  let score = 0;
  let minimumTermScore = Number.POSITIVE_INFINITY;

  for (const term of terms) {
    const termScore = Math.max(
      ...fields.map((field) => scoreTermInField(term, field)),
    );

    if (termScore <= 0) {
      return 0;
    }

    minimumTermScore = Math.min(minimumTermScore, termScore);
    score += termScore;
  }

  if (
    normalizedQuery.length > 0 &&
    fields.some((field) => field.normalized.includes(normalizedQuery))
  ) {
    score += 24;
  }

  return score + minimumTermScore * 0.25;
}

function scoreTermInField(term: string, field: SearchField) {
  const fieldScore = scoreTermAgainstText(term, field.normalized);
  const tokenScore = Math.max(
    0,
    ...field.tokens.map((token) =>
      scoreTermAgainstToken(term, token, field.kind),
    ),
  );

  return Math.max(fieldScore, tokenScore) * field.weight;
}

function scoreTermAgainstText(term: string, text: string) {
  if (text === term) {
    return 1.3;
  }

  if (text.startsWith(term)) {
    return 1.08;
  }

  if (text.includes(` ${term}`)) {
    return 1;
  }

  if (text.includes(term)) {
    return 0.86;
  }

  return 0;
}

function scoreTermAgainstToken(
  term: string,
  token: string,
  kind: SearchFieldKind,
) {
  const textScore = scoreTermAgainstTokenText(term, token);
  if (kind !== "symbol") {
    return textScore;
  }

  return Math.max(textScore, scoreSubsequenceTerm(term, token));
}

function scoreTermAgainstTokenText(term: string, token: string) {
  if (token === term) {
    return 1.22;
  }
  if (token.startsWith(term)) {
    return 1.05;
  }
  if (token.includes(term)) {
    return 0.9;
  }
  return 0;
}

function scoreSubsequenceTerm(term: string, token: string) {
  if (term.length < 3 || !isSubsequence(term, token)) {
    return 0;
  }

  return Math.max(0.42, Math.min(0.78, term.length / token.length));
}

function searchTerms(normalizedQuery: string) {
  return normalizedQuery.split(" ").filter((term) => term.length > 0);
}

function normalizeSearchText(value: string) {
  return value
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, " ")
    .trim()
    .replace(/\s+/g, " ");
}

function normalizeSuggestionText(value: string) {
  return value.toLowerCase().replace(/[^a-z0-9]+/g, "");
}

function suggestionDistanceThreshold(query: string) {
  if (query.length <= 3) {
    return 0;
  }
  if (query.length <= 7) {
    return 1;
  }
  if (query.length <= 12) {
    return 2;
  }
  return 3;
}

function editDistance(left: string, right: string, maxDistance: number) {
  if (Math.abs(left.length - right.length) > maxDistance) {
    return maxDistance + 1;
  }

  const distances = initializedDistanceMatrix(left.length, right.length);

  for (let leftIndex = 1; leftIndex <= left.length; leftIndex += 1) {
    if (fillDistanceRow(distances, left, right, leftIndex) > maxDistance) {
      return maxDistance + 1;
    }
  }

  return distances[left.length][right.length];
}

function initializedDistanceMatrix(leftLength: number, rightLength: number) {
  const distances = Array.from({ length: leftLength + 1 }, () =>
    Array.from({ length: rightLength + 1 }, () => 0),
  );

  for (const [index, row] of distances.entries()) {
    row[0] = index;
  }
  for (const [index] of distances[0].entries()) {
    distances[0][index] = index;
  }

  return distances;
}

function fillDistanceRow(
  distances: number[][],
  left: string,
  right: string,
  leftIndex: number,
) {
  let rowMin = distances[leftIndex][0];

  for (let rightIndex = 1; rightIndex <= right.length; rightIndex += 1) {
    const distance = distanceCell(
      distances,
      left,
      right,
      leftIndex,
      rightIndex,
    );
    distances[leftIndex][rightIndex] = distance;
    rowMin = Math.min(rowMin, distance);
  }

  return rowMin;
}

function distanceCell(
  distances: number[][],
  left: string,
  right: string,
  leftIndex: number,
  rightIndex: number,
) {
  const cost = left[leftIndex - 1] === right[rightIndex - 1] ? 0 : 1;
  const insert = distances[leftIndex][rightIndex - 1] + 1;
  const remove = distances[leftIndex - 1][rightIndex] + 1;
  const replace = distances[leftIndex - 1][rightIndex - 1] + cost;
  const distance = Math.min(insert, remove, replace);

  return isAdjacentTransposition(left, right, leftIndex, rightIndex)
    ? Math.min(distance, distances[leftIndex - 2][rightIndex - 2] + 1)
    : distance;
}

function isAdjacentTransposition(
  left: string,
  right: string,
  leftIndex: number,
  rightIndex: number,
) {
  return (
    leftIndex > 1 &&
    rightIndex > 1 &&
    left[leftIndex - 1] === right[rightIndex - 2] &&
    left[leftIndex - 2] === right[rightIndex - 1]
  );
}

function isSubsequence(needle: string, haystack: string) {
  let cursor = 0;

  for (const character of haystack) {
    if (character === needle[cursor]) {
      cursor += 1;

      if (cursor === needle.length) {
        return true;
      }
    }
  }

  return false;
}
