import type {
  DiagnosticCollection,
  Disposable,
  ExtensionContext,
} from "vscode";

import type { SourceDocumentProvider } from "../source";
import { registerCompletionProvider } from "./completion";
import { registerDiagnostics } from "./diagnostics";
import { registerFormattingProvider } from "./formatting";
import { registerHoverProvider } from "./hover";
import { registerInlayHintsProvider } from "./inlay-hints";
import { registerNavigationProviders } from "./navigation";
import { registerSemanticTokensProvider } from "./semantic-tokens";
import { registerSignatureHelpProvider } from "./signature-help";

export async function registerLanguageFeatures(
  context: ExtensionContext,
  diagnostics: DiagnosticCollection,
  sourceDocumentProvider: SourceDocumentProvider,
): Promise<Disposable[]> {
  return [
    registerInlayHintsProvider(context),
    registerCompletionProvider(context),
    registerHoverProvider(context),
    registerSemanticTokensProvider(context),
    registerFormattingProvider(context),
    ...(await registerDiagnostics(context, diagnostics)),
    registerSignatureHelpProvider(context),
    ...registerNavigationProviders(context, sourceDocumentProvider),
  ];
}
