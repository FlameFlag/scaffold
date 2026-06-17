/* tslint:disable */
/* eslint-disable */

export function completionItemsScaffoldScheme(): string;

export function completionItemsScaffoldSchemeForDocument(text: string, workspace_json: string): string;

export function definitionScaffoldScheme(text: string, uri: string, line: number, character: number, workspace_json: string): string;

export function diagnoseScaffoldScheme(text: string): string;

export function documentReferenceSymbolsScaffoldScheme(text: string): string;

export function formContextScaffoldScheme(text: string, line: number, character: number): string;

export function formatScaffoldScheme(text: string): string;

/**
 * Format the GitHub Flavored Markdown tables in the `doc` string.
 */
export function format_tables(doc: string): string;

export function hoverScaffoldScheme(symbol: string): string;

export function hoverScaffoldSchemeForDocument(text: string, symbol: string, workspace_json: string): string;

export function inlayHintsScaffoldScheme(text: string, start_line: number, start_character: number, end_line: number, end_character: number): string;

export function inlayHintsScaffoldSchemeForDocument(text: string, workspace_json: string, start_line: number, start_character: number, end_line: number, end_character: number): string;

export function missingDocStubScaffoldScheme(name: string, indent: string): string;

export function referenceCapabilitiesScaffoldScheme(): string;

export function referenceCatalogSchemaScaffoldScheme(): string;

export function referenceEntriesScaffoldScheme(): string;

export function referenceEntriesScaffoldSchemeForDocument(text: string, uri: string, workspace_json: string): string;

export function referenceEntriesScaffoldSchemeForWorkspace(workspace_json: string): string;

export function referenceLocationsScaffoldScheme(symbol: string, workspace_json: string): string;

export function searchReferenceEntriesScaffoldScheme(query: string, limit: number): string;

export function searchReferenceEntriesScaffoldSchemeForWorkspace(query: string, workspace_json: string, limit: number): string;

export function semanticTokensScaffoldScheme(text: string): string;

export function semanticTokensScaffoldSchemeForDocument(text: string, workspace_json: string): string;

export function signatureHelpScaffoldScheme(symbol: string): string;

export function signatureHelpScaffoldSchemeForDocument(text: string, symbol: string, workspace_json: string): string;

export function suggestReferenceEntriesScaffoldScheme(query: string, limit: number): string;

export function suggestReferenceEntriesScaffoldSchemeForWorkspace(query: string, workspace_json: string, limit: number): string;

export function symbolAtScaffoldScheme(text: string, line: number, character: number): string;

export function workspaceSymbolsScaffoldScheme(query: string, workspace_json: string): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly completionItemsScaffoldScheme: (a: number) => void;
    readonly completionItemsScaffoldSchemeForDocument: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly definitionScaffoldScheme: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
    readonly diagnoseScaffoldScheme: (a: number, b: number, c: number) => void;
    readonly documentReferenceSymbolsScaffoldScheme: (a: number, b: number, c: number) => void;
    readonly formContextScaffoldScheme: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly formatScaffoldScheme: (a: number, b: number, c: number) => void;
    readonly hoverScaffoldScheme: (a: number, b: number, c: number) => void;
    readonly hoverScaffoldSchemeForDocument: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly inlayHintsScaffoldScheme: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly inlayHintsScaffoldSchemeForDocument: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
    readonly missingDocStubScaffoldScheme: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly referenceCapabilitiesScaffoldScheme: (a: number) => void;
    readonly referenceCatalogSchemaScaffoldScheme: (a: number) => void;
    readonly referenceEntriesScaffoldScheme: (a: number) => void;
    readonly referenceEntriesScaffoldSchemeForDocument: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly referenceEntriesScaffoldSchemeForWorkspace: (a: number, b: number, c: number) => void;
    readonly referenceLocationsScaffoldScheme: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly searchReferenceEntriesScaffoldScheme: (a: number, b: number, c: number, d: number) => void;
    readonly searchReferenceEntriesScaffoldSchemeForWorkspace: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly semanticTokensScaffoldScheme: (a: number, b: number, c: number) => void;
    readonly semanticTokensScaffoldSchemeForDocument: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly signatureHelpScaffoldScheme: (a: number, b: number, c: number) => void;
    readonly signatureHelpScaffoldSchemeForDocument: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly suggestReferenceEntriesScaffoldScheme: (a: number, b: number, c: number, d: number) => void;
    readonly suggestReferenceEntriesScaffoldSchemeForWorkspace: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly symbolAtScaffoldScheme: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly workspaceSymbolsScaffoldScheme: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly format_tables: (a: number, b: number, c: number) => void;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
    readonly __wbindgen_export: (a: number, b: number, c: number) => void;
    readonly __wbindgen_export2: (a: number, b: number) => number;
    readonly __wbindgen_export3: (a: number, b: number, c: number, d: number) => number;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
