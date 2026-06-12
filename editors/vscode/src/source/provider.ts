import {
  type ExtensionContext,
  type TextDocumentContentProvider,
  Uri,
  workspace,
} from "vscode";

const sourceScheme = "scaffold-source";
const bundledSourcesFile = "scaffold-sources.json";

type BundledSources = Record<string, string>;

export class SourceDocumentProvider implements TextDocumentContentProvider {
  private bundledSources: Promise<BundledSources> | undefined;

  constructor(private readonly context: ExtensionContext) {}

  async provideTextDocumentContent(uri: Uri): Promise<string> {
    const sourcePath = sourcePathFromUri(uri);
    const source = (await this.sources())[sourcePath];
    return source ?? `; Embedded Scaffold source not found: ${sourcePath}\n`;
  }

  async uriForSource(source: string): Promise<Uri | undefined> {
    if (!(await this.hasSource(source))) {
      return undefined;
    }
    return Uri.from({
      scheme: sourceScheme,
      path: `/${source}`,
    });
  }

  private async hasSource(source: string): Promise<boolean> {
    return Object.hasOwn(await this.sources(), source);
  }

  private async sources(): Promise<BundledSources> {
    this.bundledSources ??= this.loadSources();
    return this.bundledSources;
  }

  private async loadSources(): Promise<BundledSources> {
    try {
      const bytes = await workspace.fs.readFile(
        Uri.joinPath(this.context.extensionUri, bundledSourcesFile),
      );
      return JSON.parse(new TextDecoder().decode(bytes)) as BundledSources;
    } catch {
      return {};
    }
  }
}

function sourcePathFromUri(uri: Uri): string {
  return uri.path.replace(/^\/+/, "");
}
