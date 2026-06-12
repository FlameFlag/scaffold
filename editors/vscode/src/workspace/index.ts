import { type Uri, workspace } from "vscode";

interface WorkspaceSchemeDocument {
  uri: Uri;
  text: string;
}

interface WasmWorkspaceDocument {
  uri: string;
  text: string;
}

const schemeGlob = "**/*.scm";
const excludedSchemeGlob =
  "{**/target/**,**/node_modules/**,**/.git/**,**/out/**}";

async function workspaceSchemeDocuments(): Promise<WorkspaceSchemeDocument[]> {
  const documents = new Map<string, WorkspaceSchemeDocument>();
  addOpenSchemeDocuments(documents);
  await addFileSchemeDocuments(documents);
  return [...documents.values()];
}

function addOpenSchemeDocuments(
  documents: Map<string, WorkspaceSchemeDocument>,
): void {
  for (const document of workspace.textDocuments) {
    if (document.languageId === "scaffold-scheme" && !document.isClosed) {
      documents.set(document.uri.toString(), {
        uri: document.uri,
        text: document.getText(),
      });
    }
  }
}

async function addFileSchemeDocuments(
  documents: Map<string, WorkspaceSchemeDocument>,
): Promise<void> {
  const decoder = new TextDecoder();
  for (const uri of await workspace.findFiles(schemeGlob, excludedSchemeGlob)) {
    if (!documents.has(uri.toString())) {
      documents.set(uri.toString(), {
        uri,
        text: decoder.decode(await workspace.fs.readFile(uri)),
      });
    }
  }
}

async function wasmWorkspaceDocuments(): Promise<WasmWorkspaceDocument[]> {
  return (await workspaceSchemeDocuments()).map((document) => ({
    uri: document.uri.toString(),
    text: document.text,
  }));
}

export async function wasmWorkspaceJson(): Promise<string> {
  return JSON.stringify(await wasmWorkspaceDocuments());
}
