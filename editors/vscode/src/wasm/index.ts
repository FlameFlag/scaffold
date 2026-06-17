import { type ExtensionContext, Uri, workspace } from "vscode";
import * as scaffoldWasmModule from "../../wasm/scaffold_wasm.js";

export type ScaffoldWasm = typeof scaffoldWasmModule;

let wasm: Promise<ScaffoldWasm> | undefined;

export async function scaffoldWasm(
  context: ExtensionContext,
): Promise<ScaffoldWasm> {
  wasm ??= loadScaffoldWasm(context);
  return wasm;
}

async function loadScaffoldWasm(
  context: ExtensionContext,
): Promise<ScaffoldWasm> {
  await scaffoldWasmModule.default({
    module_or_path: await workspace.fs.readFile(
      Uri.joinPath(context.extensionUri, "wasm", "scaffold_wasm_bg.wasm"),
    ),
  });
  return scaffoldWasmModule;
}
