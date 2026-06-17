const vscode = require("vscode");

async function run() {
  const extension = vscode.extensions.getExtension("scaffold.scaffold-scheme");
  assert(extension, "extension is registered");

  const workspace = vscode.workspace.workspaceFolders?.[0]?.uri;
  assert(workspace, "test workspace is open");

  const acmeUri = vscode.Uri.joinPath(workspace, "acme.scm");
  const mainUri = vscode.Uri.joinPath(workspace, "main.scm");
  const invalidUri = vscode.Uri.joinPath(workspace, "invalid.scm");
  const missingDocUri = vscode.Uri.joinPath(workspace, "missing-doc.scm");
  const mainDocument = await vscode.workspace.openTextDocument(mainUri);
  await vscode.window.showTextDocument(mainDocument);
  await waitFor(() => extension.isActive, "extension activation");
  await vscode.languages.setTextDocumentLanguage(mainDocument, "plaintext");

  const completions = await vscode.commands.executeCommand(
    "vscode.executeCompletionItemProvider",
    mainUri,
    new vscode.Position(2, 2),
  );
  assert(
    completions?.items?.some((item) => item.label === "acme-tool"),
    "workspace-backed completion",
  );

  const hovers = await vscode.commands.executeCommand(
    "vscode.executeHoverProvider",
    mainUri,
    new vscode.Position(2, 2),
  );
  assert(
    hovers?.some((hover) =>
      hover.contents.some((content) =>
        String(content.value ?? content).includes("Create an Acme"),
      ),
    ),
    "workspace-backed hover",
  );

  const definitions = await vscode.commands.executeCommand(
    "vscode.executeDefinitionProvider",
    mainUri,
    new vscode.Position(2, 2),
  );
  const definitionLocations = Array.isArray(definitions)
    ? definitions
    : definitions
      ? [definitions]
      : [];
  assert(
    definitionLocations.some((location) =>
      location.uri.path.endsWith("/acme.scm"),
    ),
    "workspace-backed definition",
  );

  const helperDefinitions = await vscode.commands.executeCommand(
    "vscode.executeDefinitionProvider",
    mainUri,
    new vscode.Position(4, 2),
  );
  const helperLocations = Array.isArray(helperDefinitions)
    ? helperDefinitions
    : helperDefinitions
      ? [helperDefinitions]
      : [];
  assert(
    helperLocations.some((location) => {
      return (
        location.uri.path.endsWith("/acme.scm") &&
        location.range.start.line === 13
      );
    }),
    "workspace-backed helper definition",
  );

  const localDefinitions = await vscode.commands.executeCommand(
    "vscode.executeDefinitionProvider",
    mainUri,
    new vscode.Position(8, 2),
  );
  const localLocations = Array.isArray(localDefinitions)
    ? localDefinitions
    : localDefinitions
      ? [localDefinitions]
      : [];
  assert(
    localLocations.some((location) => {
      return (
        location.uri.path.endsWith("/main.scm") &&
        location.range.start.line === 6
      );
    }),
    "local undocumented definition",
  );

  const stdlibDocument = await vscode.workspace.openTextDocument({
    content: '(catalog)\n(path/join "a" "b")\n',
    language: "scaffold-scheme",
  });
  await vscode.window.showTextDocument(stdlibDocument);
  const stdlibDefinitions = await vscode.commands.executeCommand(
    "vscode.executeDefinitionProvider",
    stdlibDocument.uri,
    new vscode.Position(0, 2),
  );
  const stdlibLocations = Array.isArray(stdlibDefinitions)
    ? stdlibDefinitions
    : stdlibDefinitions
      ? [stdlibDefinitions]
      : [];
  const catalogLocation = stdlibLocations.find((location) => {
    return (
      location.uri.scheme === "scaffold-source" &&
      location.uri.path.endsWith("/src/dsl/std/catalog/root.scm") &&
      location.range.start.line === 13
    );
  });
  assert(catalogLocation, "bundled stdlib definition");
  const catalogSource = await vscode.workspace.openTextDocument(
    catalogLocation.uri,
  );
  assert(
    catalogSource.getText().includes("(define (catalog . tools)"),
    "bundled stdlib source document",
  );

  const stdlibSymbols = await vscode.commands.executeCommand(
    "vscode.executeWorkspaceSymbolProvider",
    "path/join",
  );
  assert(
    stdlibSymbols?.some(
      (symbol) =>
        symbol.name === "path/join" &&
        symbol.location.uri.scheme === "scaffold-source" &&
        symbol.location.uri.path.endsWith("/src/dsl/std/path.scm"),
    ),
    "bundled stdlib workspace symbol",
  );

  const references = await vscode.commands.executeCommand(
    "vscode.executeReferenceProvider",
    mainUri,
    new vscode.Position(2, 2),
  );
  assert(
    references?.some((location) => location.uri.path.endsWith("/main.scm")) &&
      references.some((location) => location.uri.path.endsWith("/acme.scm")),
    "workspace-backed references",
  );

  const symbols = await vscode.commands.executeCommand(
    "vscode.executeWorkspaceSymbolProvider",
    "acme",
  );
  assert(
    symbols?.some(
      (symbol) =>
        symbol.name === "acme-tool" &&
        symbol.location.uri.path.endsWith("/acme.scm"),
    ),
    "workspace symbol",
  );
  assert(
    symbols?.some(
      (symbol) =>
        symbol.name === "acme-helper" &&
        symbol.location.uri.path.endsWith("/acme.scm"),
    ),
    "undocumented workspace symbol",
  );

  const documentSymbols = await vscode.commands.executeCommand(
    "vscode.executeDocumentSymbolProvider",
    acmeUri,
  );
  assert(
    documentSymbols?.some(
      (symbol) =>
        symbol.name === "acme-tool" &&
        symbol.detail === "(acme-tool name [mode])",
    ),
    "document symbol detail",
  );

  const hints = await vscode.commands.executeCommand(
    "vscode.executeInlayHintProvider",
    mainUri,
    new vscode.Range(new vscode.Position(0, 0), new vscode.Position(10, 0)),
  );
  assert(
    hints?.some((hint) => hint.label === "name:"),
    "workspace-backed inlay hint",
  );

  const referenceDocument = await vscode.workspace.openTextDocument(
    vscode.Uri.from({
      scheme: "scaffold-reference",
      path: "/reference.md",
    }),
  );
  assert(
    hasRenderedReferenceDocument(referenceDocument.getText()),
    "rendered virtual reference document",
  );
  await vscode.commands.executeCommand("scaffold.openLanguageReference");

  const signatureHelp = await vscode.commands.executeCommand(
    "vscode.executeSignatureHelpProvider",
    mainUri,
    new vscode.Position(2, 12),
    "(",
  );
  const signature = signatureHelp?.signatures?.[0];
  assert(
    signature?.label === "(acme-tool name [mode])",
    "workspace-backed signature help label",
  );
  assert(
    signature.parameters?.some(
      (parameter) =>
        parameter.label === "name" && parameter.documentation === "Tool name.",
    ),
    "workspace-backed signature help documented parameter",
  );
  assert(
    signature.parameters?.some(
      (parameter) =>
        parameter.label === "[mode]" && parameter.documentation === undefined,
    ),
    "workspace-backed signature help undocumented parameter",
  );

  const unformattedDocument = await vscode.workspace.openTextDocument({
    content: "(define x 1)(define y 2)",
    language: "scaffold-scheme",
  });
  await vscode.window.showTextDocument(unformattedDocument);
  const validEdits = await vscode.commands.executeCommand(
    "vscode.executeFormatDocumentProvider",
    unformattedDocument.uri,
    { tabSize: 2, insertSpaces: true },
  );
  assert(
    Array.isArray(validEdits) && validEdits.length > 0,
    "valid formatting returns edits",
  );

  const invalidDocument = await vscode.workspace.openTextDocument(invalidUri);
  await vscode.window.showTextDocument(invalidDocument);
  await waitFor(
    () => vscode.languages.getDiagnostics(invalidUri).length > 0,
    "invalid diagnostics",
  );

  const invalidEdits = await vscode.commands.executeCommand(
    "vscode.executeFormatDocumentProvider",
    invalidUri,
    { tabSize: 2, insertSpaces: true },
  );
  assert((invalidEdits?.length ?? 0) === 0, "invalid formatting has no edits");

  const missingDocDocument =
    await vscode.workspace.openTextDocument(missingDocUri);
  await vscode.window.showTextDocument(missingDocDocument);
  await waitFor(
    () =>
      vscode.languages.getDiagnostics(missingDocUri).some((diagnostic) => {
        return (
          diagnostic.code === "scaffold::dsl::missing-doc" &&
          diagnostic.data?.name === "local-helper" &&
          diagnostic.data?.line === 2
        );
      }),
    "missing-doc diagnostic data",
  );
  const missingDocDiagnostic = vscode.languages
    .getDiagnostics(missingDocUri)
    .find((diagnostic) => diagnostic.code === "scaffold::dsl::missing-doc");
  const actions = await vscode.commands.executeCommand(
    "vscode.executeCodeActionProvider",
    missingDocUri,
    missingDocDiagnostic.range,
    vscode.CodeActionKind.QuickFix.value,
  );
  const addDocAction = actions?.find(
    (action) => action.title === "Add doc stub for `local-helper`",
  );
  const edits = addDocAction?.edit?.entries?.() ?? [];
  assert(
    edits.some(([, textEdits]) =>
      textEdits.some((edit) => edit.newText.includes("(doc-next")),
    ),
    "missing-doc quick fix edit",
  );
}

exports.run = run;

function assert(condition, message) {
  if (!condition) {
    throw new Error(`Expected ${message}`);
  }
}

function hasRenderedReferenceDocument(text) {
  return (
    text.includes("| Entries | Groups | Examples | Deprecated |") &&
    text.includes("[`build`](#entry-build-1i9vvkz)") &&
    text.includes("Create a build action for tools compiled from source.")
  );
}

async function waitFor(predicate, label) {
  const deadline = Date.now() + 5000;
  while (Date.now() < deadline) {
    if (predicate()) {
      return;
    }
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
  throw new Error(`Timed out waiting for ${label}`);
}
