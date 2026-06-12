import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";
import type { ScaffoldWasmModule } from "./scaffold-module.mjs";

export async function withScaffoldWasm<T>(
  extensionRoot: string,
  run: (scaffold: ScaffoldWasmModule) => Promise<T>,
): Promise<T> {
  const wasmRoot = path.join(extensionRoot, "wasm");
  const tempDir = await mkdtemp(path.join(tmpdir(), "scaffold-wasm-smoke-"));

  try {
    const jsSource = await readFile(
      path.join(wasmRoot, "scaffold_wasm.js"),
      "utf8",
    );
    const modulePath = path.join(tempDir, "scaffold_wasm.mjs");
    await writeFile(modulePath, jsSource);

    const scaffold = (await import(
      pathToFileURL(modulePath).href
    )) as ScaffoldWasmModule;
    const wasmBytes = await readFile(
      path.join(wasmRoot, "scaffold_wasm_bg.wasm"),
    );
    await scaffold.default({ module_or_path: wasmBytes });

    return await run(scaffold);
  } finally {
    await rm(tempDir, { force: true, recursive: true });
  }
}
