import {
  type Disposable,
  DocumentSymbol,
  type ExtensionContext,
  Location,
  languages,
  Position,
  Range,
  SymbolInformation,
  SymbolKind,
  SymbolTag,
  type TextDocument,
  Uri,
} from "vscode";

import { schemeSelector } from "../scheme";
import type { SourceDocumentProvider } from "../source";
import { type ScaffoldWasm, scaffoldWasm } from "../wasm";
import { wasmWorkspaceJson } from "../workspace";

interface WasmSymbolRange {
  line: number;
  start: number;
  length: number;
}

interface WasmDocumentSymbol extends WasmSymbolRange {
  name: string;
  detail?: string | null;
  kind: "function" | "keyword";
}

interface WasmDefinitionLocation extends WasmSymbolRange {
  uri: string;
}

interface WasmWorkspaceSymbol extends WasmSymbolRange {
  name: string;
  kind: "function" | "keyword";
  group?: string;
  deprecated: boolean;
  uri: string;
}

export function registerNavigationProviders(
  context: ExtensionContext,
  sourceDocumentProvider: SourceDocumentProvider,
): Disposable[] {
  return [
    languages.registerDefinitionProvider(schemeSelector, {
      async provideDefinition(document, position) {
        const scaffold = await scaffoldWasm(context);
        const symbol = scaffold.symbolAtScaffoldScheme(
          document.getText(),
          position.line,
          position.character,
        );
        if (!symbol) {
          return undefined;
        }
        const definition = JSON.parse(
          scaffold.definitionScaffoldScheme(
            document.getText(),
            document.uri.toString(),
            position.line,
            position.character,
            await wasmWorkspaceJson(),
          ),
        ) as WasmDefinitionLocation | null;
        if (!definition) {
          return undefined;
        }
        return new Location(
          await uriForDefinition(definition.uri, sourceDocumentProvider),
          rangeForSymbol(definition),
        );
      },
    }),
    languages.registerReferenceProvider(schemeSelector, {
      async provideReferences(document, position) {
        const scaffold = await scaffoldWasm(context);
        const symbol = scaffold.symbolAtScaffoldScheme(
          document.getText(),
          position.line,
          position.character,
        );
        if (!symbol) {
          return [];
        }
        return referencesForSymbol(
          scaffold,
          symbol,
          await wasmWorkspaceJson(),
          sourceDocumentProvider,
        );
      },
    }),
    languages.registerDocumentSymbolProvider(schemeSelector, {
      async provideDocumentSymbols(document) {
        return documentSymbols(await scaffoldWasm(context), document).map(
          toDocumentSymbol,
        );
      },
    }),
    languages.registerWorkspaceSymbolProvider({
      async provideWorkspaceSymbols(query) {
        const symbols = JSON.parse(
          (await scaffoldWasm(context)).workspaceSymbolsScaffoldScheme(
            query,
            await wasmWorkspaceJson(),
          ),
        ) as WasmWorkspaceSymbol[];
        return Promise.all(
          symbols.map((symbol) =>
            toWorkspaceSymbol(symbol, sourceDocumentProvider),
          ),
        );
      },
    }),
  ];
}

async function referencesForSymbol(
  scaffold: ScaffoldWasm,
  symbol: string,
  workspaceJson: string,
  sourceDocumentProvider: SourceDocumentProvider,
): Promise<Location[]> {
  const locations = JSON.parse(
    scaffold.referenceLocationsScaffoldScheme(symbol, workspaceJson),
  ) as WasmDefinitionLocation[];
  return Promise.all(
    locations.map(async (location) => {
      return new Location(
        await uriForDefinition(location.uri, sourceDocumentProvider),
        rangeForSymbol(location),
      );
    }),
  );
}

function documentSymbols(
  scaffold: ScaffoldWasm,
  document: TextDocument,
): WasmDocumentSymbol[] {
  return JSON.parse(
    scaffold.documentReferenceSymbolsScaffoldScheme(document.getText()),
  ) as WasmDocumentSymbol[];
}

function toDocumentSymbol(symbol: WasmDocumentSymbol): DocumentSymbol {
  return new DocumentSymbol(
    symbol.name,
    symbol.detail ?? "",
    symbol.kind === "keyword" ? SymbolKind.Key : SymbolKind.Function,
    rangeForSymbol(symbol),
    rangeForSymbol(symbol),
  );
}

function rangeForSymbol(symbol: WasmSymbolRange): Range {
  return new Range(
    new Position(symbol.line, symbol.start),
    new Position(symbol.line, symbol.start + symbol.length),
  );
}

async function toWorkspaceSymbol(
  symbol: WasmWorkspaceSymbol,
  sourceDocumentProvider: SourceDocumentProvider,
): Promise<SymbolInformation> {
  const information = new SymbolInformation(
    symbol.name,
    symbol.kind === "keyword" ? SymbolKind.Key : SymbolKind.Function,
    symbol.group ?? "",
    new Location(
      await uriForDefinition(symbol.uri, sourceDocumentProvider),
      rangeForSymbol(symbol),
    ),
  );
  if (symbol.deprecated) {
    information.tags = [SymbolTag.Deprecated];
  }
  return information;
}

async function uriForDefinition(
  uri: string,
  sourceDocumentProvider: SourceDocumentProvider,
): Promise<Uri> {
  return (await sourceDocumentProvider.uriForSource(uri)) ?? Uri.parse(uri);
}
