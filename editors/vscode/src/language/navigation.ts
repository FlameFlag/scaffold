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
import { parseWasmJson } from "../wasm/json";
import { wasmWorkspaceJson } from "../workspace";
import {
  isWasmDefinitionLocation,
  isWasmDocumentSymbol,
  isWasmWorkspaceSymbol,
  type WasmDefinitionLocation,
  type WasmDocumentSymbol,
  type WasmWorkspaceSymbol,
} from "./navigation-data";

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
        const definition = parseWasmJson<WasmDefinitionLocation | null>(
          scaffold.definitionScaffoldScheme(
            document.getText(),
            document.uri.toString(),
            position.line,
            position.character,
            await wasmWorkspaceJson(),
          ),
          null,
        );
        if (!isWasmDefinitionLocation(definition)) {
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
        const symbols = parseWasmJson<WasmWorkspaceSymbol[]>(
          (await scaffoldWasm(context)).workspaceSymbolsScaffoldScheme(
            query,
            await wasmWorkspaceJson(),
          ),
          [],
        );
        return Promise.all(
          symbols
            .filter(isWasmWorkspaceSymbol)
            .map((symbol) => toWorkspaceSymbol(symbol, sourceDocumentProvider)),
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
  const locations = parseWasmJson<WasmDefinitionLocation[]>(
    scaffold.referenceLocationsScaffoldScheme(symbol, workspaceJson),
    [],
  );
  return Promise.all(
    locations.filter(isWasmDefinitionLocation).map(async (location) => {
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
  return parseWasmJson<WasmDocumentSymbol[]>(
    scaffold.documentReferenceSymbolsScaffoldScheme(document.getText()),
    [],
  ).filter(isWasmDocumentSymbol);
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

function rangeForSymbol(
  symbol: WasmDefinitionLocation | WasmDocumentSymbol | WasmWorkspaceSymbol,
): Range {
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
