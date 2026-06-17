import {
  commands,
  type Event,
  EventEmitter,
  type QuickPickItem,
  type TextDocumentContentProvider,
  Uri,
  window,
  workspace,
} from "vscode";

import {
  renderMissingReferenceEntryMarkdown,
  renderReferenceDocumentMarkdown,
  renderReferenceEntryMarkdown,
} from "./markdown";
import type { ReferenceProvider } from "./provider";
import type { ReferenceEntry } from "./types";
import { referenceEntryNameFromPath, referenceEntryPath } from "./uri";

const referenceScheme = "scaffold-reference";
const referenceSearchLimit = 50;
const referenceSearchPlaceholder = "Search Scaffold reference docs";
const indexUri = Uri.from({
  scheme: referenceScheme,
  path: "/reference.md",
});

interface ReferencePick extends QuickPickItem {
  entry: ReferenceEntry;
}

type ReferenceQueryResult = {
  entries: ReferenceEntry[];
  suggested: boolean;
};

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

    return missingReferenceEntryMarkdown(uri);
  }

  uriForEntry(entry: ReferenceEntry): Uri {
    const uri = Uri.from({
      scheme: referenceScheme,
      path: referenceEntryPath(entry.name),
    });
    this.knownUris.set(uri.toString(), uri);
    return uri;
  }

  private async entryForUri(uri: Uri): Promise<ReferenceEntry | undefined> {
    const name = referenceEntryNameFromPath(uri.path);
    if (!name) {
      return undefined;
    }

    return (await this.referenceProvider.entries()).find(
      (entry) => entry.name === name,
    );
  }
}

function missingReferenceEntryMarkdown(uri: Uri): string {
  return renderMissingReferenceEntryMarkdown(
    referenceEntryNameFromPath(uri.path),
  );
}

export async function openLanguageReference(): Promise<void> {
  await openMarkdownPreview(indexUri);
}

export async function searchReference(
  referenceProvider: ReferenceProvider,
  referenceDocumentProvider: ReferenceDocumentProvider,
): Promise<void> {
  const selected = await pickReferenceEntry(referenceProvider);

  if (selected) {
    await openReferenceEntry(selected, referenceDocumentProvider);
  }
}

async function pickReferenceEntry(
  referenceProvider: ReferenceProvider,
): Promise<ReferenceEntry | undefined> {
  const quickPick = window.createQuickPick<ReferencePick>();
  quickPick.placeholder = referenceSearchPlaceholder;
  quickPick.matchOnDescription = false;
  quickPick.matchOnDetail = false;

  let request = 0;
  async function updateItems(query: string): Promise<void> {
    const currentRequest = ++request;
    quickPick.busy = true;
    quickPick.placeholder = "Loading Scaffold reference docs...";
    try {
      const result = await entriesForQuery(referenceProvider, query);
      if (currentRequest === request) {
        quickPick.items = referencePicks(result.entries);
        quickPick.placeholder = referenceSearchPlaceholderForResults(
          query,
          result.entries.length,
          result.suggested,
        );
      }
    } catch (error) {
      if (currentRequest === request) {
        quickPick.items = [];
        quickPick.placeholder = "Reference docs could not be loaded";
        void window.showErrorMessage(referenceSearchErrorMessage(error));
      }
    } finally {
      if (currentRequest === request) {
        quickPick.busy = false;
      }
    }
  }

  await updateItems("");
  return new Promise<ReferenceEntry | undefined>((resolve) => {
    let selected: ReferenceEntry | undefined;
    const disposables = [
      quickPick.onDidChangeValue((query) => {
        void updateItems(query);
      }),
      quickPick.onDidAccept(() => {
        selected = quickPick.selectedItems[0]?.entry;
        quickPick.hide();
      }),
      quickPick.onDidHide(() => {
        for (const disposable of disposables) {
          disposable.dispose();
        }
        quickPick.dispose();
        resolve(selected);
      }),
    ];
    quickPick.show();
  });
}

function referenceSearchPlaceholderForResults(
  query: string,
  entryCount: number,
  suggested: boolean,
): string {
  if (entryCount > 0 && suggested) {
    return "No direct matches; showing suggestions";
  }
  if (entryCount > 0 || query.trim().length === 0) {
    return referenceSearchPlaceholder;
  }

  return "No reference entries match this search";
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

function referencePicks(entries: ReferenceEntry[]): ReferencePick[] {
  return entries.map((entry) => ({
    label: entry.name,
    description: entry.group,
    detail: referencePickDetail(entry),
    entry,
  }));
}

async function entriesForQuery(
  referenceProvider: ReferenceProvider,
  query: string,
): Promise<ReferenceQueryResult> {
  if (!query.trim()) {
    return {
      entries: await referenceProvider.entries(),
      suggested: false,
    };
  }

  const entries = await referenceProvider.searchEntries(
    query,
    referenceSearchLimit,
  );
  if (entries.length > 0) {
    return { entries, suggested: false };
  }

  return {
    entries: await referenceProvider.suggestEntries(query, 5),
    suggested: true,
  };
}

function referenceSearchErrorMessage(error: unknown): string {
  const reason = error instanceof Error ? error.message : String(error);
  return `Unable to load Scaffold reference docs: ${reason}`;
}

function referencePickDetail(entry: ReferenceEntry): string | undefined {
  const details = [];
  if (entry.summary) {
    details.push(entry.summary);
  }
  const source = entry.source_location ?? entry.source;
  if (source) {
    details.push(`Source: ${source}`);
  }
  return details.length > 0 ? details.join("\n") : undefined;
}
