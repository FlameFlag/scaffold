export type ScaffoldWasmModule = {
  default: (options: { module_or_path: Uint8Array }) => Promise<void>;
  completionItemsScaffoldSchemeForDocument: (
    text: string,
    workspace: string,
  ) => string;
  hoverScaffoldSchemeForDocument: (
    text: string,
    symbol: string,
    workspace: string,
  ) => string;
  definitionScaffoldScheme: (
    text: string,
    uri: string,
    line: number,
    character: number,
    workspace: string,
  ) => string;
  diagnoseScaffoldScheme: (text: string) => string;
  missingDocStubScaffoldScheme: (name: string, indent: string) => string;
  formatScaffoldScheme: (text: string) => string;
  workspaceSymbolsScaffoldScheme: (query: string, workspace: string) => string;
  referenceLocationsScaffoldScheme: (
    symbol: string,
    workspace: string,
  ) => string;
  documentReferenceSymbolsScaffoldScheme: (text: string) => string;
  signatureHelpScaffoldSchemeForDocument: (
    text: string,
    symbol: string,
    workspace: string,
  ) => string;
  formContextScaffoldScheme: (
    text: string,
    line: number,
    character: number,
  ) => string;
  semanticTokensScaffoldSchemeForDocument: (
    text: string,
    workspace: string,
  ) => string;
  referenceCatalogSchemaScaffoldScheme: () => string;
};
