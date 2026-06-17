import {
  type Disposable,
  type ExtensionContext,
  languages,
  MarkdownString,
  ParameterInformation,
  SignatureHelp,
  SignatureInformation,
} from "vscode";

import { schemeSelector } from "../scheme";
import { scaffoldWasm } from "../wasm";
import { parseWasmJson } from "../wasm/json";
import { wasmWorkspaceJson } from "../workspace";
import {
  type FormContext,
  isFormContext,
  isWasmSignatureHelp,
  type WasmSignatureHelp,
} from "./signature-help-data";

export function registerSignatureHelpProvider(
  context: ExtensionContext,
): Disposable {
  return languages.registerSignatureHelpProvider(
    schemeSelector,
    {
      async provideSignatureHelp(document, position) {
        const scaffold = await scaffoldWasm(context);
        const form = parseWasmJson<FormContext | null>(
          scaffold.formContextScaffoldScheme(
            document.getText(),
            position.line,
            position.character,
          ),
          null,
        );
        if (!isFormContext(form)) {
          return undefined;
        }
        const help = parseWasmJson<WasmSignatureHelp | null>(
          scaffold.signatureHelpScaffoldSchemeForDocument(
            document.getText(),
            form.head,
            await wasmWorkspaceJson(),
          ),
          null,
        );
        if (!isWasmSignatureHelp(help)) {
          return undefined;
        }
        return signatureHelp(help, form.active_argument);
      },
    },
    {
      triggerCharacters: [" ", "("],
      retriggerCharacters: [" "],
    },
  );
}

function signatureHelp(
  item: WasmSignatureHelp,
  activeArgument: number,
): SignatureHelp {
  const help = new SignatureHelp();
  const signature = new SignatureInformation(
    item.label,
    new MarkdownString(item.documentation),
  );
  signature.parameters = item.parameters.map(
    (param) =>
      new ParameterInformation(param.label, param.documentation ?? undefined),
  );
  help.signatures = [signature];
  help.activeSignature = 0;
  help.activeParameter =
    signature.parameters.length === 0
      ? 0
      : Math.min(activeArgument, signature.parameters.length - 1);
  return help;
}
