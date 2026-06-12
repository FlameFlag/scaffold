import { type ExtensionContext, Uri, workspace } from "vscode";
import * as scaffoldWasmModule from "../../wasm/scaffold_wasm.js";

export interface ScaffoldWasm {
  completionItemsScaffoldScheme(): string;
  completionItemsScaffoldSchemeForDocument(
    text: string,
    workspaceJson: string,
  ): string;
  default(
    input?: Uint8Array | { module_or_path: Uint8Array },
  ): Promise<unknown>;
  definitionScaffoldScheme(
    text: string,
    uri: string,
    line: number,
    character: number,
    workspaceJson: string,
  ): string;
  diagnoseScaffoldScheme(text: string): string;
  documentReferenceSymbolsScaffoldScheme(text: string): string;
  formContextScaffoldScheme(
    text: string,
    line: number,
    character: number,
  ): string;
  formatScaffoldScheme(text: string): string;
  hoverScaffoldScheme(symbol: string): string;
  hoverScaffoldSchemeForDocument(
    text: string,
    symbol: string,
    workspaceJson: string,
  ): string;
  inlayHintsScaffoldScheme(
    text: string,
    startLine: number,
    startCharacter: number,
    endLine: number,
    endCharacter: number,
  ): string;
  inlayHintsScaffoldSchemeForDocument(
    text: string,
    workspaceJson: string,
    startLine: number,
    startCharacter: number,
    endLine: number,
    endCharacter: number,
  ): string;
  missingDocStubScaffoldScheme(name: string, indent: string): string;
  referenceCapabilitiesScaffoldScheme(): string;
  referenceCatalogSchemaScaffoldScheme(): string;
  referenceEntriesScaffoldScheme(): string;
  referenceEntriesScaffoldSchemeForDocument(
    text: string,
    uri: string,
    workspaceJson: string,
  ): string;
  referenceEntriesScaffoldSchemeForWorkspace(workspaceJson: string): string;
  referenceLocationsScaffoldScheme(
    symbol: string,
    workspaceJson: string,
  ): string;
  semanticTokensScaffoldScheme(text: string): string;
  semanticTokensScaffoldSchemeForDocument(
    text: string,
    workspaceJson: string,
  ): string;
  signatureHelpScaffoldScheme(symbol: string): string;
  signatureHelpScaffoldSchemeForDocument(
    text: string,
    symbol: string,
    workspaceJson: string,
  ): string;
  symbolAtScaffoldScheme(text: string, line: number, character: number): string;
  workspaceSymbolsScaffoldScheme(query: string, workspaceJson: string): string;
}

let wasm: Promise<ScaffoldWasm> | undefined;

export async function scaffoldWasm(
  context: ExtensionContext,
): Promise<ScaffoldWasm> {
  wasm ??= loadScaffoldWasm(context);
  return wasm;
}

async function loadScaffoldWasm(
  context: ExtensionContext,
): Promise<ScaffoldWasm> {
  const module = scaffoldWasmModule as unknown as ScaffoldWasm;
  await module.default({
    module_or_path: await workspace.fs.readFile(
      Uri.joinPath(context.extensionUri, "wasm", "scaffold_wasm_bg.wasm"),
    ),
  });
  return module;
}
