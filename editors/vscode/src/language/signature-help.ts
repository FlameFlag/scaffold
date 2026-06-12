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
import { wasmWorkspaceJson } from "../workspace";

interface FormContext {
  head: string;
  active_argument: number;
}

interface WasmSignatureHelp {
  label: string;
  documentation: string;
  parameters: WasmSignatureParameter[];
}

interface WasmSignatureParameter {
  label: string;
  documentation?: string | null;
}

export function registerSignatureHelpProvider(
  context: ExtensionContext,
): Disposable {
  return languages.registerSignatureHelpProvider(
    schemeSelector,
    {
      async provideSignatureHelp(document, position) {
        const scaffold = await scaffoldWasm(context);
        const form = JSON.parse(
          scaffold.formContextScaffoldScheme(
            document.getText(),
            position.line,
            position.character,
          ),
        ) as FormContext | null;
        if (!form) {
          return undefined;
        }
        const help = JSON.parse(
          scaffold.signatureHelpScaffoldSchemeForDocument(
            document.getText(),
            form.head,
            await wasmWorkspaceJson(),
          ),
        ) as WasmSignatureHelp | null;
        if (!help) {
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
