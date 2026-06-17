import {
  type Disposable,
  type ExtensionContext,
  InlayHint,
  InlayHintKind,
  languages,
  Position,
} from "vscode";

import { schemeSelector } from "../scheme";
import { scaffoldWasm } from "../wasm";
import { parseWasmJson } from "../wasm/json";
import { wasmWorkspaceJson } from "../workspace";
import { isWasmInlayHint, type WasmInlayHint } from "./inlay-hints-data";

export function registerInlayHintsProvider(
  context: ExtensionContext,
): Disposable {
  return languages.registerInlayHintsProvider(schemeSelector, {
    async provideInlayHints(document, range) {
      return parseWasmJson<WasmInlayHint[]>(
        (await scaffoldWasm(context)).inlayHintsScaffoldSchemeForDocument(
          document.getText(),
          await wasmWorkspaceJson(),
          range.start.line,
          range.start.character,
          range.end.line,
          range.end.character,
        ),
        [],
      )
        .filter(isWasmInlayHint)
        .map(toVsCodeInlayHint);
    },
  });
}

function toVsCodeInlayHint(item: WasmInlayHint): InlayHint {
  const hint = new InlayHint(
    new Position(item.line, item.start),
    item.label,
    InlayHintKind.Parameter,
  );
  hint.tooltip = item.tooltip;
  hint.paddingRight = true;
  return hint;
}
