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
import { wasmWorkspaceJson } from "../workspace";

interface WasmInlayHint {
  line: number;
  start: number;
  label: string;
  tooltip: string;
}

export function registerInlayHintsProvider(
  context: ExtensionContext,
): Disposable {
  return languages.registerInlayHintsProvider(schemeSelector, {
    async provideInlayHints(document, range) {
      return (
        JSON.parse(
          (await scaffoldWasm(context)).inlayHintsScaffoldSchemeForDocument(
            document.getText(),
            await wasmWorkspaceJson(),
            range.start.line,
            range.start.character,
            range.end.line,
            range.end.character,
          ),
        ) as WasmInlayHint[]
      ).map(toVsCodeInlayHint);
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
