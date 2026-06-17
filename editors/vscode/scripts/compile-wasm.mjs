import { resolve } from "node:path";
import { extensionRoot } from "./script-paths.mjs";
import { buildScaffoldWasm, generateWasmBindings } from "./wasm-build.mjs";

await buildScaffoldWasm({ cwd: extensionRoot });
await generateWasmBindings(resolve(extensionRoot, "wasm"), {
  cwd: extensionRoot,
});
