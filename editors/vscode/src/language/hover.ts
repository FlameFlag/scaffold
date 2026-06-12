import {
  type Disposable,
  type ExtensionContext,
  Hover,
  languages,
  MarkdownString,
} from "vscode";

import { schemeSelector } from "../scheme";
import { scaffoldWasm } from "../wasm";
import { wasmWorkspaceJson } from "../workspace";

export function registerHoverProvider(context: ExtensionContext): Disposable {
  return languages.registerHoverProvider(schemeSelector, {
    async provideHover(document, position) {
      const scaffold = await scaffoldWasm(context);
      const symbol = scaffold.symbolAtScaffoldScheme(
        document.getText(),
        position.line,
        position.character,
      );
      if (!symbol) {
        return undefined;
      }
      const markdown = scaffold.hoverScaffoldSchemeForDocument(
        document.getText(),
        symbol,
        await wasmWorkspaceJson(),
      );
      if (!markdown.trim()) {
        return undefined;
      }
      return new Hover(new MarkdownString(markdown));
    },
  });
}
