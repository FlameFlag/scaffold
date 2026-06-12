import {
  commands,
  type DiagnosticCollection,
  type Disposable,
  type ExtensionContext,
  languages,
  window,
  workspace,
} from "vscode";

import { registerLanguageFeatures } from "../language";
import {
  openLanguageReference,
  openReferenceEntry,
  ReferenceDocumentProvider,
  type ReferenceEntry,
  ReferenceProvider,
  searchReference,
} from "../reference";
import { isSchemeDocument } from "../scheme";
import { SourceDocumentProvider } from "../source";

let diagnostics: DiagnosticCollection | undefined;

export async function activate(context: ExtensionContext): Promise<void> {
  diagnostics = languages.createDiagnosticCollection("scaffold");
  const referenceProvider = new ReferenceProvider(context);
  const referenceDocumentProvider = new ReferenceDocumentProvider(
    referenceProvider,
  );
  const sourceDocumentProvider = new SourceDocumentProvider(context);

  context.subscriptions.push(
    diagnostics,
    window.registerTreeDataProvider("scaffold.reference", referenceProvider),
    workspace.registerTextDocumentContentProvider(
      "scaffold-reference",
      referenceDocumentProvider,
    ),
    workspace.registerTextDocumentContentProvider(
      "scaffold-source",
      sourceDocumentProvider,
    ),
    ...(await registerLanguageFeatures(
      context,
      diagnostics,
      sourceDocumentProvider,
    )),
    ...registerReferenceInvalidation(
      referenceProvider,
      referenceDocumentProvider,
    ),
    commands.registerCommand("scaffold.formatDocument", async () => {
      await commands.executeCommand("editor.action.formatDocument");
    }),
    commands.registerCommand("scaffold.openLanguageReference", async () => {
      await openLanguageReference();
    }),
    commands.registerCommand("scaffold.refreshReference", () => {
      refreshReference(referenceProvider, referenceDocumentProvider);
    }),
    commands.registerCommand("scaffold.searchReference", async () => {
      await searchReference(referenceProvider, referenceDocumentProvider);
    }),
    commands.registerCommand(
      "scaffold.openReferenceEntry",
      async (entry: ReferenceEntry) => {
        await openReferenceEntry(entry, referenceDocumentProvider);
      },
    ),
  );
}

export function deactivate(): void {
  diagnostics?.dispose();
  diagnostics = undefined;
}

function registerReferenceInvalidation(
  referenceProvider: ReferenceProvider,
  referenceDocumentProvider: ReferenceDocumentProvider,
): Disposable[] {
  const watcher = workspace.createFileSystemWatcher("**/*.scm");
  return [
    watcher,
    watcher.onDidCreate(() =>
      refreshReference(referenceProvider, referenceDocumentProvider),
    ),
    watcher.onDidChange(() =>
      refreshReference(referenceProvider, referenceDocumentProvider),
    ),
    watcher.onDidDelete(() =>
      refreshReference(referenceProvider, referenceDocumentProvider),
    ),
    workspace.onDidOpenTextDocument((document) => {
      if (isSchemeDocument(document)) {
        refreshReference(referenceProvider, referenceDocumentProvider);
      }
    }),
    workspace.onDidChangeTextDocument((event) => {
      if (isSchemeDocument(event.document)) {
        refreshReference(referenceProvider, referenceDocumentProvider);
      }
    }),
    workspace.onDidCloseTextDocument((document) => {
      if (isSchemeDocument(document)) {
        refreshReference(referenceProvider, referenceDocumentProvider);
      }
    }),
  ];
}

function refreshReference(
  referenceProvider: ReferenceProvider,
  referenceDocumentProvider: ReferenceDocumentProvider,
): void {
  referenceProvider.refresh();
  referenceDocumentProvider.refresh();
}
