import {
  type Disposable,
  type ExtensionContext,
  languages,
  Position,
  Range,
  type TextDocument,
  TextEdit,
} from "vscode";

import { schemeSelector } from "../scheme";
import { type ScaffoldWasm, scaffoldWasm } from "../wasm";

export function registerFormattingProvider(
  context: ExtensionContext,
): Disposable {
  return languages.registerDocumentFormattingEditProvider(schemeSelector, {
    async provideDocumentFormattingEdits(document) {
      const formatted = formatDocumentOrUndefined(
        await scaffoldWasm(context),
        document.getText(),
      );
      if (formatted === undefined || formatted === document.getText()) {
        return [];
      }
      return [TextEdit.replace(fullDocumentRange(document), formatted)];
    },
  });
}

function formatDocumentOrUndefined(
  scaffold: ScaffoldWasm,
  text: string,
): string | undefined {
  try {
    return scaffold.formatScaffoldScheme(text);
  } catch {
    return undefined;
  }
}

function fullDocumentRange(document: TextDocument): Range {
  return new Range(
    new Position(0, 0),
    document.lineAt(document.lineCount - 1).range.end,
  );
}
