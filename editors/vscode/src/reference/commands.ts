import {
  commands,
  type Event,
  EventEmitter,
  type TextDocumentContentProvider,
  Uri,
  window,
  workspace,
} from "vscode";

import {
  renderReferenceDocumentMarkdown,
  renderReferenceEntryMarkdown,
} from "./markdown";
import type { ReferenceProvider } from "./provider";
import type { ReferenceEntry } from "./types";

const referenceScheme = "scaffold-reference";
const indexUri = Uri.from({
  scheme: referenceScheme,
  path: "/reference.md",
});

export class ReferenceDocumentProvider implements TextDocumentContentProvider {
  private readonly changed = new EventEmitter<Uri>();
  private readonly knownUris = new Map<string, Uri>([
    [indexUri.toString(), indexUri],
  ]);
  readonly onDidChange: Event<Uri> = this.changed.event;

  constructor(private readonly referenceProvider: ReferenceProvider) {}

  refresh(): void {
    for (const uri of this.knownUris.values()) {
      this.changed.fire(uri);
    }
  }

  async provideTextDocumentContent(uri: Uri): Promise<string> {
    this.knownUris.set(uri.toString(), uri);
    if (uri.path === indexUri.path) {
      return renderReferenceDocumentMarkdown(
        await this.referenceProvider.entries(),
      );
    }

    const entry = await this.entryForUri(uri);
    if (entry) {
      return renderReferenceEntryMarkdown(entry);
    }

    return "# Scaffold Scheme Reference\n\nReference entry not found.\n";
  }

  uriForEntry(entry: ReferenceEntry): Uri {
    const uri = Uri.from({
      scheme: referenceScheme,
      path: `/entries/${encodeURIComponent(entry.name)}.md`,
    });
    this.knownUris.set(uri.toString(), uri);
    return uri;
  }

  private async entryForUri(uri: Uri): Promise<ReferenceEntry | undefined> {
    const match = /^\/entries\/(.+)\.md$/.exec(uri.path);
    if (!match) {
      return undefined;
    }

    const name = decodeURIComponent(match[1]);
    return (await this.referenceProvider.entries()).find(
      (entry) => entry.name === name,
    );
  }
}

export async function openLanguageReference(): Promise<void> {
  await openMarkdownPreview(indexUri);
}

export async function searchReference(
  referenceProvider: ReferenceProvider,
  referenceDocumentProvider: ReferenceDocumentProvider,
): Promise<void> {
  const selected = await window.showQuickPick(
    (await referenceProvider.entries()).map((entry) => ({
      label: entry.name,
      description: entry.group,
      detail: entry.summary,
      entry,
    })),
    {
      matchOnDescription: true,
      matchOnDetail: true,
      placeHolder: "Search Scaffold Scheme docs",
    },
  );
  if (selected) {
    await openReferenceEntry(selected.entry, referenceDocumentProvider);
  }
}

export async function openReferenceEntry(
  entry: ReferenceEntry,
  referenceDocumentProvider: ReferenceDocumentProvider,
): Promise<void> {
  await openMarkdownPreview(referenceDocumentProvider.uriForEntry(entry));
}

async function openMarkdownPreview(uri: Uri): Promise<void> {
  await workspace.openTextDocument(uri);
  await commands.executeCommand("markdown.showPreviewToSide", uri);
}
