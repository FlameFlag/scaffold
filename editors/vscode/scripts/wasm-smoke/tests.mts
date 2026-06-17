import { assert } from "./assert.mjs";
import { documentText, workspace, workspaceDocuments } from "./fixtures.mjs";
import type { ScaffoldWasmModule } from "./scaffold-module.mjs";

type LabeledItem = {
  label?: string;
};

type LocationItem = {
  uri?: string;
  line?: number;
};

type DiagnosticItem = {
  code?: string;
  data?: {
    name?: string;
    line?: number;
  };
};

type SymbolItem = {
  name?: string;
  detail?: string;
};

type SignatureHelp = {
  label?: string;
  parameters: Array<{
    label?: string;
    documentation?: string | null;
  }>;
} | null;

type FormContext = {
  head?: string;
  active_argument?: number;
} | null;

type SemanticToken = {
  text?: string;
  token_type?: string;
};

type CatalogSchema = {
  title?: string;
  objects: Array<{
    name?: string;
  }>;
};

type ReferenceEntry = {
  name?: string;
  effect?: string | null;
  requires_capability?: string[];
};

export function runWasmSmokeTest(scaffold: ScaffoldWasmModule): void {
  assertWorkspaceCompletion(scaffold);
  assertWorkspaceHover(scaffold);
  assertDefinitions(scaffold);
  assertDiagnostics(scaffold);
  assertFormatting(scaffold);
  assertWorkspaceSymbols(scaffold);
  assertDocumentSymbols(scaffold);
  assertSignatureHelp(scaffold);
  assertSemanticTokens(scaffold);
  assertReferenceEntries(scaffold);
  assertCatalogSchema(scaffold);
}

function assertWorkspaceCompletion(scaffold: ScaffoldWasmModule): void {
  const completions = parseJson<LabeledItem[]>(
    scaffold.completionItemsScaffoldSchemeForDocument(documentText, workspace),
  );

  assert(
    completions.some((item) => item.label === "acme-tool"),
    "workspace completion",
  );
}

function assertWorkspaceHover(scaffold: ScaffoldWasmModule): void {
  const hover = scaffold.hoverScaffoldSchemeForDocument(
    documentText,
    "acme-tool",
    workspace,
  );

  assert(hover.includes("Acme."), "workspace hover");
  assert(hover.includes("**Parameters**"), "workspace hover parameters label");
  assert(
    hover.includes("| `name`") && hover.includes("Name docs."),
    "workspace hover parameter table",
  );
  assert(!hover.includes("Parameters:"), "workspace hover avoids old labels");
}

function assertDefinitions(scaffold: ScaffoldWasmModule): void {
  assertWorkspaceDefinition(scaffold);
  assertUndocumentedWorkspaceDefinition(scaffold);
  assertLocalDefinition(scaffold);
}

function assertWorkspaceDefinition(scaffold: ScaffoldWasmModule): void {
  const definition = parseJson<LocationItem | null>(
    scaffold.definitionScaffoldScheme(
      documentText,
      "file:///workspace/main.scm",
      1,
      2,
      workspace,
    ),
  );
  assert(
    definition?.uri === "file:///workspace/acme.scm",
    "workspace definition",
  );
}

function assertUndocumentedWorkspaceDefinition(
  scaffold: ScaffoldWasmModule,
): void {
  const undocumentedDefinition = parseJson<LocationItem | null>(
    scaffold.definitionScaffoldScheme(
      documentText,
      "file:///workspace/main.scm",
      2,
      2,
      workspace,
    ),
  );
  assert(
    undocumentedDefinition?.uri === "file:///workspace/acme.scm",
    "undocumented workspace definition",
  );
}

function assertLocalDefinition(scaffold: ScaffoldWasmModule): void {
  const localDefinition = parseJson<LocationItem | null>(
    scaffold.definitionScaffoldScheme(
      documentText,
      "file:///workspace/main.scm",
      4,
      2,
      workspace,
    ),
  );
  assert(
    localDefinition?.uri === "file:///workspace/main.scm" &&
      localDefinition.line === 3,
    "undocumented local definition",
  );
}

function assertDiagnostics(scaffold: ScaffoldWasmModule): void {
  const diagnostics = parseJson<DiagnosticItem[]>(
    scaffold.diagnoseScaffoldScheme("(define x 1"),
  );
  assert(
    diagnostics.some((item) => item.code === "scaffold::dsl::syntax"),
    "syntax diagnostic",
  );

  const missingDocDiagnostics = parseJson<DiagnosticItem[]>(
    scaffold.diagnoseScaffoldScheme(
      '(tool #:name "demo")\n(define (local-helper value) value)',
    ),
  );
  assert(
    missingDocDiagnostics.some(
      (item) => item.data?.name === "local-helper" && item.data?.line === 1,
    ),
    "missing-doc diagnostic data",
  );

  const missingDocStub = scaffold.missingDocStubScaffoldScheme(
    "local-helper",
    "  ",
  );
  assert(
    missingDocStub ===
      '  (doc-next\n    (summary "Describe `local-helper`."))\n\n',
    "missing-doc quick fix stub",
  );
}

function assertFormatting(scaffold: ScaffoldWasmModule): void {
  const formatted = scaffold.formatScaffoldScheme("(define x 1)(define y 2)");
  assert(formatted === "(define x 1)\n\n(define y 2)\n", "formatter output");
}

function assertWorkspaceSymbols(scaffold: ScaffoldWasmModule): void {
  const symbols = parseJson<SymbolItem[]>(
    scaffold.workspaceSymbolsScaffoldScheme("acme", workspace),
  );
  assert(
    symbols.some((item) => item.name === "acme-tool"),
    "workspace symbols",
  );
  assert(
    symbols.some((item) => item.name === "acme-helper"),
    "undocumented workspace symbols",
  );

  const referenceLocations = parseJson<LocationItem[]>(
    scaffold.referenceLocationsScaffoldScheme("acme-tool", workspace),
  );
  assert(
    referenceLocations.some(
      (item) => item.uri === "file:///workspace/acme.scm",
    ),
    "workspace reference locations",
  );
}

function assertDocumentSymbols(scaffold: ScaffoldWasmModule): void {
  const documentSymbols = parseJson<SymbolItem[]>(
    scaffold.documentReferenceSymbolsScaffoldScheme(workspaceDocuments[0].text),
  );
  assert(
    documentSymbols.some(
      (item) =>
        item.name === "acme-tool" && item.detail === "(acme-tool name [mode])",
    ),
    "document symbol detail",
  );
}

function assertSignatureHelp(scaffold: ScaffoldWasmModule): void {
  const signature = parseJson<SignatureHelp>(
    scaffold.signatureHelpScaffoldSchemeForDocument(
      documentText,
      "acme-tool",
      workspace,
    ),
  );
  assert(
    signature?.label === "(acme-tool name [mode])",
    "workspace signature help label",
  );
  assert(
    signature.parameters.some(
      (item) => item.label === "name" && item.documentation === "Name docs.",
    ),
    "workspace signature help documented parameter",
  );
  assert(
    signature.parameters.some(
      (item) => item.label === "[mode]" && item.documentation === null,
    ),
    "workspace signature help undocumented parameter",
  );

  const formContext = parseJson<FormContext>(
    scaffold.formContextScaffoldScheme(documentText, 1, 16),
  );
  assert(
    formContext?.head === "acme-tool" && formContext.active_argument === 0,
    "signature form context",
  );
}

function assertSemanticTokens(scaffold: ScaffoldWasmModule): void {
  const semanticTokens = parseJson<SemanticToken[]>(
    scaffold.semanticTokensScaffoldSchemeForDocument(documentText, workspace),
  );
  assert(
    semanticTokens.some(
      (item) => item.text === "acme-tool" && item.token_type === "function",
    ),
    "workspace semantic token",
  );
}

function assertReferenceEntries(scaffold: ScaffoldWasmModule): void {
  const entries = parseJson<ReferenceEntry[]>(
    scaffold.referenceEntriesScaffoldScheme(),
  );
  const sourcePath = entries.find((item) => item.name === "source/path");
  assert(
    sourcePath?.effect === "context-read-only" &&
      sourcePath.requires_capability?.includes("scaffold.workspace"),
    "reference entry effect and required capability metadata",
  );

  const unrelatedSearch = parseJson<ReferenceEntry[]>(
    scaffold.searchReferenceEntriesScaffoldScheme("nope", 5),
  );
  assert(unrelatedSearch.length === 0, "reference search rejects noise");

  const typoSearch = parseJson<ReferenceEntry[]>(
    scaffold.searchReferenceEntriesScaffoldScheme("ctlg tool", 5),
  );
  assert(
    typoSearch.some((item) => item.name === "catalog/tool"),
    "reference search keeps useful typo matching",
  );

  const typoSuggestions = parseJson<ReferenceEntry[]>(
    scaffold.suggestReferenceEntriesScaffoldScheme("catlgtool", 5),
  );
  assert(
    typoSuggestions[0]?.name === "catalog/tool",
    "reference suggestions recover compact symbol typos",
  );
}

function assertCatalogSchema(scaffold: ScaffoldWasmModule): void {
  const schema = parseJson<CatalogSchema>(
    scaffold.referenceCatalogSchemaScaffoldScheme(),
  );
  assert(schema.title === "Scaffold Catalog", "catalog schema title");
  assert(
    schema.objects.some((item) => item.name === "tool"),
    "catalog schema tool object",
  );
}

function parseJson<T>(text: string): T {
  return JSON.parse(text) as T;
}
