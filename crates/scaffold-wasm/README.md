# scaffold-wasm

WASM-facing editor analysis entry points for Scaffold Scheme.

This crate owns the pure editor-analysis core used by the VS Code browser
extension. It does not spawn `scaffold lsp`, perform native filesystem I/O, or
depend on Tokio/tower-lsp transport. Browser hosts gather open/workspace Scheme
documents through the VS Code web extension APIs and pass that text into these
WASM functions, keeping the Rust core self-hostable on `wasm32-unknown-unknown`.

Current export:

- `formatScaffoldScheme(text: string): string`
- `diagnoseScaffoldScheme(text: string): string`
- `missingDocStubScaffoldScheme(name: string, indent: string): string`
- `semanticTokensScaffoldScheme(text: string): string`
- `semanticTokensScaffoldSchemeForDocument(text: string, workspaceJson: string): string`
- `completionItemsScaffoldScheme(): string`
- `completionItemsScaffoldSchemeForDocument(text: string, workspaceJson: string): string`
- `hoverScaffoldScheme(symbol: string): string`
- `hoverScaffoldSchemeForDocument(text: string, symbol: string, workspaceJson: string): string`
- `signatureHelpScaffoldScheme(symbol: string): string`
- `signatureHelpScaffoldSchemeForDocument(text: string, symbol: string, workspaceJson: string): string`
- `referenceEntriesScaffoldScheme(): string`
- `searchReferenceEntriesScaffoldScheme(query: string, limit: number): string`
- `suggestReferenceEntriesScaffoldScheme(query: string, limit: number): string`
- `referenceCapabilitiesScaffoldScheme(): string`
- `referenceCatalogSchemaScaffoldScheme(): string`
- `referenceEntriesScaffoldSchemeForWorkspace(workspaceJson: string): string`
- `searchReferenceEntriesScaffoldSchemeForWorkspace(query: string, workspaceJson: string, limit: number): string`
- `suggestReferenceEntriesScaffoldSchemeForWorkspace(query: string, workspaceJson: string, limit: number): string`
- `referenceEntriesScaffoldSchemeForDocument(text: string, uri: string, workspaceJson: string): string`
- `symbolAtScaffoldScheme(text: string, line: number, character: number): string`
- `formContextScaffoldScheme(text: string, line: number, character: number): string`
- `referenceLocationsScaffoldScheme(symbol: string, workspaceJson: string): string`
- `documentReferenceSymbolsScaffoldScheme(text: string): string`
- `inlayHintsScaffoldScheme(text: string, startLine: number, startCharacter: number, endLine: number, endCharacter: number): string`
- `inlayHintsScaffoldSchemeForDocument(text: string, workspaceJson: string, startLine: number, startCharacter: number, endLine: number, endCharacter: number): string`
- `definitionScaffoldScheme(text: string, uri: string, line: number, character: number, workspaceJson: string): string`
- `workspaceSymbolsScaffoldScheme(query: string, workspaceJson: string): string`

The VS Code extension's `browser` entry point loads this module and uses it for
document formatting, syntax diagnostics, semantic highlighting, completions, and
hover documentation in web extension hosts where spawning `scaffold lsp` is not
available. Browser reference tree/search and parameter inlay hints use the same
generated bundled reference data plus workspace documents passed in from the
extension. Workspace-aware exports use `scaffold-docs` for the same
definition-aware `(doc ...)` extraction as the native LSP: bundled docs are the
base, imported workspace libraries are added according to the current
document's `(import ...)` graph, and current-document `(doc ...)` entries
override non-keyword bundled docs.

Refresh the bundled base reference after changing built-in `(doc ...)` sources
with:

```text
bun run wasm:reference
```

Use `bun run wasm:reference:check` to verify that the committed reference JSON
matches the current built-in docs without rewriting files.

Build with:

```text
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --locked --version 0.2.123
bun run --cwd editors/vscode compile:check
bun run --cwd editors/vscode compile:wasm
bun run --cwd editors/vscode compile
bun run --cwd editors/vscode test:wasm:smoke
bun run --cwd editors/vscode test:web
```

The `compile:check` command verifies that the committed VS Code source bundle
and WASM bindings match what the current Rust and generated reference sources
would produce. The `compile:wasm` command builds the Rust crate in release mode
with normalized path prefixes and runs `wasm-bindgen` to emit
`editors/vscode/wasm/scaffold_wasm.js`,
`editors/vscode/wasm/scaffold_wasm.d.ts`, and
`editors/vscode/wasm/scaffold_wasm_bg.wasm`.
The smoke test initializes the generated web-target bundle from raw WASM bytes
and exercises workspace-aware completion, hover, definition, diagnostics,
formatting, and workspace symbols.
The VS Code browser extension is bundled with esbuild into `out/browser.js` so
the web extension host does not need to resolve relative CommonJS modules.
`test:web` runs the bundled extension in a headless Chromium VS Code web host
and checks workspace-backed completion, hover, definition, workspace symbols,
inlay hints, formatting, diagnostics, and invalid-formatting no-op behavior.
