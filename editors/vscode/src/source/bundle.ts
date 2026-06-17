export type BundledSources = Record<string, string>;

export type SourceBundle = {
  sources: BundledSources;
  error: string | null;
};

export function parseBundledSources(text: string): BundledSources {
  const value = JSON.parse(text) as unknown;

  if (!isStringRecord(value)) {
    throw new Error("expected source bundle to be an object of path strings");
  }

  return value;
}

export function sourceBundleLoadFailure(
  error: unknown,
  label = "scaffold-sources.json",
): SourceBundle {
  return {
    sources: {},
    error: `Could not load ${label}: ${errorMessage(error)}`,
  };
}

export function sourceBundleContent(
  bundle: SourceBundle,
  sourcePath: string,
): string {
  if (Object.hasOwn(bundle.sources, sourcePath)) {
    return bundle.sources[sourcePath];
  }

  if (bundle.error && isEmbeddedScaffoldSourcePath(sourcePath)) {
    return [
      `; Embedded Scaffold source bundle could not be loaded.`,
      `; Requested source: ${sourcePath}`,
      `; ${bundle.error}`,
      "",
    ].join("\n");
  }

  return `; Embedded Scaffold source not found: ${sourcePath}\n`;
}

export function shouldOpenSourceBundleUri(
  bundle: SourceBundle,
  sourcePath: string,
): boolean {
  return (
    Object.hasOwn(bundle.sources, sourcePath) ||
    (bundle.error !== null && isEmbeddedScaffoldSourcePath(sourcePath))
  );
}

function isStringRecord(value: unknown): value is Record<string, string> {
  return (
    typeof value === "object" &&
    value !== null &&
    !Array.isArray(value) &&
    Object.values(value).every((item) => typeof item === "string")
  );
}

function isEmbeddedScaffoldSourcePath(sourcePath: string): boolean {
  return sourcePath.startsWith("src/") && sourcePath.endsWith(".scm");
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
