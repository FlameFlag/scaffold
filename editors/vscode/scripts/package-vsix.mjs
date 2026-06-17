#!/usr/bin/env node
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { parseArgs } from "node:util";

const require = createRequire(import.meta.url);
const { pack } = require("@vscode/vsce/out/package");

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const extensionRoot = path.resolve(scriptDir, "..");

function usage() {
  console.error(
    "Usage: bun scripts/package-vsix.mjs [--out <path> | --out-dir <directory>]",
  );
}

function packageArgs(argv) {
  const { values } = parseArgs({
    args: argv,
    options: {
      out: { type: "string" },
      "out-dir": { type: "string" },
    },
    strict: true,
  });
  const out = values.out;
  const outDir = values["out-dir"] ?? (out === undefined ? "out" : undefined);
  validateOutputArgs(out, outDir);
  return { out, outDir };
}

function validateOutputArgs(out, outDir) {
  if (out && outDir) {
    usage();
    throw new Error("Use either --out or --out-dir, not both.");
  }
  if (out === "" || outDir === "") {
    usage();
    throw new Error("Output path cannot be empty.");
  }
}

function resolveFromExtensionRoot(value) {
  return path.isAbsolute(value) ? value : path.resolve(extensionRoot, value);
}

export async function packageVsix(argv = process.argv.slice(2)) {
  const { out, outDir } = packageArgs(argv);
  const packagePath = resolveFromExtensionRoot(outDir ?? out);

  if (outDir) {
    fs.mkdirSync(packagePath, { recursive: true });
  } else {
    fs.mkdirSync(path.dirname(packagePath), { recursive: true });
  }

  return pack({
    cwd: extensionRoot,
    packagePath,
    useYarn: false,
    dependencies: false,
    allowMissingRepository: true,
  });
}

if (
  process.argv[1] &&
  import.meta.url === pathToFileURL(process.argv[1]).href
) {
  const { packagePath: writtenPath, files } = await packageVsix();
  console.log(`Packaged: ${writtenPath} (${files.length} files)`);
}
