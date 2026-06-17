const entryPathPattern = /^\/entries\/(.+)\.md$/;

export function referenceEntryPath(name: string): string {
  return `/entries/${encodeURIComponent(name)}.md`;
}

export function referenceEntryNameFromPath(path: string): string | undefined {
  const match = entryPathPattern.exec(path);
  if (!match) {
    return undefined;
  }

  return decodeUriComponent(match[1]);
}

function decodeUriComponent(value: string): string | undefined {
  try {
    return decodeURIComponent(value);
  } catch {
    return undefined;
  }
}
