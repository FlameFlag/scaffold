export function sameMarkdownParagraph(left, right) {
  return normalizeMarkdownParagraph(left) === normalizeMarkdownParagraph(right);
}

function normalizeMarkdownParagraph(value) {
  return String(value ?? "")
    .split(/\s+/)
    .filter(Boolean)
    .join(" ");
}
