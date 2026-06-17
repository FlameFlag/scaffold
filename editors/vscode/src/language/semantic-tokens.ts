import {
  type Disposable,
  type ExtensionContext,
  languages,
  Position,
  Range,
  SemanticTokensBuilder,
  SemanticTokensLegend,
} from "vscode";

import { schemeSelector } from "../scheme";
import { scaffoldWasm } from "../wasm";
import { parseWasmJson } from "../wasm/json";
import { wasmWorkspaceJson } from "../workspace";
import {
  isWasmSemanticToken,
  semanticTokenModifierNames,
  semanticTokenTypeNames,
  type WasmSemanticToken,
} from "./semantic-tokens-data";

const semanticTokensLegend = new SemanticTokensLegend(
  [...semanticTokenTypeNames],
  [...semanticTokenModifierNames],
);

export function registerSemanticTokensProvider(
  context: ExtensionContext,
): Disposable {
  return languages.registerDocumentSemanticTokensProvider(
    schemeSelector,
    {
      async provideDocumentSemanticTokens(document) {
        const tokens = parseWasmJson<WasmSemanticToken[]>(
          (await scaffoldWasm(context)).semanticTokensScaffoldSchemeForDocument(
            document.getText(),
            await wasmWorkspaceJson(),
          ),
          [],
        );
        const builder = new SemanticTokensBuilder(semanticTokensLegend);
        for (const token of tokens.filter(isWasmSemanticToken)) {
          builder.push(
            new Range(
              new Position(token.line, token.start),
              new Position(token.line, token.start + token.length),
            ),
            token.token_type,
            token.modifiers,
          );
        }
        return builder.build();
      },
    },
    semanticTokensLegend,
  );
}
