#!/usr/bin/env bun
import { chmod, readFile, rename, rm, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(scriptDir, "..", "..", "..");
const referenceJsonPath = resolve(
  repoRoot,
  "crates/scaffold-wasm/src/reference.json",
);
const referenceMinJsonPath = resolve(
  repoRoot,
  "crates/scaffold-wasm/src/reference.min.json",
);
const referenceJsonTemp = `${referenceJsonPath}.${process.pid}.${Date.now()}.tmp`;
const referenceMinJsonTemp = `${referenceMinJsonPath}.${process.pid}.${Date.now()}.tmp`;
const checkOnly = scriptArgs().check;

try {
  await run([
    "cargo",
    "run",
    "--locked",
    "--manifest-path",
    resolve(repoRoot, "Cargo.toml"),
    "-p",
    "scaffold",
    "--",
    "docs",
    "--format",
    "json",
    "--output",
    referenceJsonTemp,
  ]);

  const referenceJson = await readFile(referenceJsonTemp, "utf8");
  const referenceMinJson = JSON.stringify(JSON.parse(referenceJson));

  if (checkOnly) {
    await checkGeneratedFile(referenceJsonPath, referenceJson, "reference.json");
    await checkGeneratedFile(
      referenceMinJsonPath,
      referenceMinJson,
      "reference.min.json",
    );
  } else {
    await writeFile(referenceMinJsonTemp, referenceMinJson);
    await rename(referenceJsonTemp, referenceJsonPath);
    await rename(referenceMinJsonTemp, referenceMinJsonPath);
    await Promise.all([
      chmod(referenceJsonPath, 0o644),
      chmod(referenceMinJsonPath, 0o644),
    ]);
    console.log(`Generated WASM reference at ${referenceJsonPath}`);
  }
} finally {
  await rm(referenceJsonTemp, { force: true });
  await rm(referenceMinJsonTemp, { force: true });
}

async function checkGeneratedFile(path, expected, label) {
  const current = await readFile(path, "utf8").catch(() => null);
  if (current !== expected) {
    throw new Error(
      `crates/scaffold-wasm/src/${label} is stale; run "bun run wasm:reference"`,
    );
  }
}

async function run(argv) {
  const proc = Bun.spawn(argv, {
    cwd: repoRoot,
    stdout: "inherit",
    stderr: "inherit",
  });
  const exitCode = await proc.exited;

  if (exitCode !== 0) {
    throw new Error(`${argv.slice(0, 2).join(" ")} failed with ${exitCode}`);
  }
}

function scriptArgs() {
  return parseArgs({
    options: {
      check: { type: "boolean", default: false },
    },
    strict: true,
  }).values;
}
