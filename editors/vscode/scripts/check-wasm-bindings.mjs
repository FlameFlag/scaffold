import { mkdtemp, readdir, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { basename, resolve } from "node:path";
import { extensionRoot } from "./script-paths.mjs";
import { buildScaffoldWasm, generateWasmBindings } from "./wasm-build.mjs";

const tempDir = await mkdtemp(resolve(tmpdir(), "scaffold-wasm-bindings-"));
const generatedDir = resolve(tempDir, "wasm");
const expectedDir = resolve(extensionRoot, "wasm");

try {
  await buildScaffoldWasm({ cwd: extensionRoot });
  await generateWasmBindings(generatedDir, { cwd: extensionRoot });

  const [expectedFiles, generatedFiles] = await Promise.all([
    generatedBindingFiles(expectedDir),
    generatedBindingFiles(generatedDir),
  ]);
  if (expectedFiles.join("\n") !== generatedFiles.join("\n")) {
    throw new Error(
      `WASM binding file set is stale; expected ${expectedFiles.join(", ")}, generated ${generatedFiles.join(", ")}. Run "bun run --cwd editors/vscode compile:wasm"`,
    );
  }

  const stale = [];
  for (const file of expectedFiles) {
    const [expected, generated] = await Promise.all([
      readFile(resolve(expectedDir, file)),
      readFile(resolve(generatedDir, file)),
    ]);
    if (!expected.equals(generated)) {
      stale.push(file);
    }
  }

  if (stale.length > 0) {
    throw new Error(
      `${stale.map((file) => basename(file)).join(", ")} stale; run "bun run --cwd editors/vscode compile:wasm"`,
    );
  }
} finally {
  await rm(tempDir, { recursive: true, force: true });
}

async function generatedBindingFiles(dir) {
  return (await readdir(dir))
    .filter((file) => file.startsWith("scaffold_wasm"))
    .sort();
}
