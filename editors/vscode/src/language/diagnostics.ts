import {
  CodeAction,
  CodeActionKind,
  Diagnostic,
  type DiagnosticCollection,
  DiagnosticSeverity,
  type Disposable,
  type ExtensionContext,
  languages,
  Position,
  Range,
  type TextDocument,
  WorkspaceEdit,
  workspace,
} from "vscode";

import { isSchemeDocument, schemeSelector } from "../scheme";
import { scaffoldWasm } from "../wasm";
import { parseWasmJson } from "../wasm/json";
import {
  diagnosticLineNumber,
  isWasmDiagnostic,
  missingDocCode,
  type WasmDiagnostic,
} from "./diagnostics-data";

type ScaffoldDiagnostic = Diagnostic & {
  data?: WasmDiagnostic["data"];
};

export async function registerDiagnostics(
  context: ExtensionContext,
  diagnostics: DiagnosticCollection,
): Promise<Disposable[]> {
  await Promise.all(
    workspace.textDocuments.map((document) =>
      updateDiagnostics(context, diagnostics, document),
    ),
  );
  return [
    languages.registerCodeActionsProvider(
      schemeSelector,
      {
        provideCodeActions(document, _range, actionContext) {
          return Promise.all(
            actionContext.diagnostics
              .filter(isMissingDocDiagnostic)
              .map((diagnostic) =>
                missingDocCodeAction(context, document, diagnostic),
              ),
          ).then((actions) =>
            actions.filter(
              (action): action is CodeAction => action !== undefined,
            ),
          );
        },
      },
      { providedCodeActionKinds: [CodeActionKind.QuickFix] },
    ),
    workspace.onDidOpenTextDocument((document) => {
      void updateDiagnostics(context, diagnostics, document);
    }),
    workspace.onDidChangeTextDocument((event) => {
      void updateDiagnostics(context, diagnostics, event.document);
    }),
    workspace.onDidCloseTextDocument((document) => {
      diagnostics.delete(document.uri);
    }),
  ];
}

async function updateDiagnostics(
  context: ExtensionContext,
  diagnostics: DiagnosticCollection,
  document: TextDocument,
): Promise<void> {
  if (!isSchemeDocument(document)) {
    return;
  }
  diagnostics.set(
    document.uri,
    parseWasmJson<WasmDiagnostic[]>(
      (await scaffoldWasm(context)).diagnoseScaffoldScheme(document.getText()),
      [],
    )
      .filter(isWasmDiagnostic)
      .map((diagnostic) => toVsCodeDiagnostic(document, diagnostic)),
  );
}

function isMissingDocDiagnostic(diagnostic: Diagnostic): boolean {
  return (
    diagnostic.code === missingDocCode ||
    (typeof diagnostic.code === "object" &&
      diagnostic.code.value === missingDocCode)
  );
}

async function missingDocCodeAction(
  context: ExtensionContext,
  document: TextDocument,
  diagnostic: Diagnostic,
): Promise<CodeAction | undefined> {
  const diagnosticData = (diagnostic as ScaffoldDiagnostic).data;
  const name = missingDocName(document, diagnostic, diagnosticData);
  if (!name) {
    return undefined;
  }
  const line = missingDocLine(document, diagnostic, diagnosticData);
  if (!line) {
    return undefined;
  }
  const action = new CodeAction(
    `Add doc stub for \`${name}\``,
    CodeActionKind.QuickFix,
  );
  action.diagnostics = [diagnostic];
  action.isPreferred = true;
  const edit = new WorkspaceEdit();
  edit.insert(
    document.uri,
    new Position(line.lineNumber, 0),
    await missingDocStub(context, name, line.text),
  );
  action.edit = edit;
  return action;
}

function missingDocName(
  document: TextDocument,
  diagnostic: Diagnostic,
  diagnosticData: WasmDiagnostic["data"] | undefined,
): string | undefined {
  const name =
    diagnosticData?.name ?? document.getText(diagnostic.range).trim();
  return name || undefined;
}

function missingDocLine(
  document: TextDocument,
  diagnostic: Diagnostic,
  diagnosticData: WasmDiagnostic["data"] | undefined,
) {
  const line = diagnosticLineNumber(
    diagnosticData,
    diagnostic.range.start.line,
    document.lineCount,
  );
  return line === undefined ? undefined : document.lineAt(line);
}

async function missingDocStub(
  context: ExtensionContext,
  name: string,
  lineText: string,
): Promise<string> {
  return (await scaffoldWasm(context)).missingDocStubScaffoldScheme(
    name,
    lineText.match(/^\s*/)?.[0] ?? "",
  );
}

function toVsCodeDiagnostic(
  document: TextDocument,
  diagnostic: WasmDiagnostic,
): Diagnostic {
  const item = new Diagnostic(
    new Range(
      document.positionAt(diagnostic.offset),
      document.positionAt(diagnostic.offset + Math.max(diagnostic.length, 1)),
    ),
    diagnostic.message,
    diagnosticSeverity(diagnostic),
  );
  item.source = "scaffold";
  item.code = diagnostic.code;
  (item as ScaffoldDiagnostic).data = diagnostic.data;
  return item;
}

function diagnosticSeverity(diagnostic: WasmDiagnostic): DiagnosticSeverity {
  switch (diagnostic.severity) {
    case "warning":
      return DiagnosticSeverity.Warning;
    case "information":
      return DiagnosticSeverity.Information;
    case "hint":
      return DiagnosticSeverity.Hint;
    default:
      return DiagnosticSeverity.Error;
  }
}
