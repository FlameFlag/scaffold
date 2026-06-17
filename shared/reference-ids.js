export function anchor(value) {
  return value
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "");
}

export function entryId(value) {
  return `entry-${anchor(value)}-${stableIdSuffix(value)}`;
}

export function groupId(value) {
  return `group-${anchor(value)}`;
}

function stableIdSuffix(value) {
  let hash = 2_166_136_261;
  for (const character of value) {
    hash ^= character.codePointAt(0) ?? 0;
    hash = Math.imul(hash, 16_777_619) >>> 0;
  }
  return hash.toString(36);
}
