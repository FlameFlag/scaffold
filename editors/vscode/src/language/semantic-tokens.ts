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
import { wasmWorkspaceJson } from "../workspace";

const semanticTokensLegend = new SemanticTokensLegend(
  ["function", "keyword", "string", "comment", "parameter"],
  ["defaultLibrary", "documentation", "deprecated", "definition"],
);

interface WasmSemanticToken {
  text: string;
  line: number;
  start: number;
  length: number;
  token_type: string;
  modifiers: string[];
}

export function registerSemanticTokensProvider(
  context: ExtensionContext,
): Disposable {
  return languages.registerDocumentSemanticTokensProvider(
    schemeSelector,
    {
      async provideDocumentSemanticTokens(document) {
        const tokens = JSON.parse(
          (await scaffoldWasm(context)).semanticTokensScaffoldSchemeForDocument(
            document.getText(),
            await wasmWorkspaceJson(),
          ),
        ) as WasmSemanticToken[];
        const builder = new SemanticTokensBuilder(semanticTokensLegend);
        for (const token of tokens) {
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
