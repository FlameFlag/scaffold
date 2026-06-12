import path from "node:path";
import { runWasmSmokeTest } from "./wasm-smoke/tests.mjs";
import { withScaffoldWasm } from "./wasm-smoke/wasm-loader.mjs";

const extensionRoot = path.resolve(import.meta.dirname, "../..");

await withScaffoldWasm(extensionRoot, async (scaffold) => {
  runWasmSmokeTest(scaffold);
});

console.log("wasm smoke ok");
