import {
  type ExtensionContext,
  type TextDocumentContentProvider,
  Uri,
  workspace,
} from "vscode";

import {
  parseBundledSources,
  type SourceBundle,
  shouldOpenSourceBundleUri,
  sourceBundleContent,
  sourceBundleLoadFailure,
} from "./bundle";

const sourceScheme = "scaffold-source";
const bundledSourcesFile = "scaffold-sources.json";

export class SourceDocumentProvider implements TextDocumentContentProvider {
  private sourceBundle: Promise<SourceBundle> | undefined;

  constructor(private readonly context: ExtensionContext) {}

  async provideTextDocumentContent(uri: Uri): Promise<string> {
    const sourcePath = sourcePathFromUri(uri);
    return sourceBundleContent(await this.bundle(), sourcePath);
  }

  async uriForSource(source: string): Promise<Uri | undefined> {
    if (!shouldOpenSourceBundleUri(await this.bundle(), source)) {
      return undefined;
    }
    return Uri.from({
      scheme: sourceScheme,
      path: `/${source}`,
    });
  }

  private async bundle(): Promise<SourceBundle> {
    this.sourceBundle ??= this.loadBundle();
    return this.sourceBundle;
  }

  private async loadBundle(): Promise<SourceBundle> {
    try {
      const bytes = await workspace.fs.readFile(
        Uri.joinPath(this.context.extensionUri, bundledSourcesFile),
      );
      return {
        sources: parseBundledSources(new TextDecoder().decode(bytes)),
        error: null,
      };
    } catch (error) {
      return sourceBundleLoadFailure(error, bundledSourcesFile);
    }
  }
}

function sourcePathFromUri(uri: Uri): string {
  return uri.path.replace(/^\/+/, "");
}
