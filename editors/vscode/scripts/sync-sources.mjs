import { readdir, readFile, rename, rm, writeFile } from "node:fs/promises";
import { dirname, join, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";

const extensionRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const repoRoot = resolve(extensionRoot, "..", "..");
const dslRoot = join(repoRoot, "crates", "scaffold-dsl", "src");
const outputPath = join(extensionRoot, "scaffold-sources.json");
const temporaryOutputPath = `${outputPath}.${process.pid}.${Date.now()}.tmp`;
const checkOnly = scriptArgs().check;

const sources = Object.create(null);

await addSources(join(dslRoot, "std"), "src/dsl/std");
await addSources(join(dslRoot, "extensions"), "src/extensions");

const orderedSources = Object.fromEntries(
  Object.entries(sources).sort(([left], [right]) => left.localeCompare(right)),
);
const output = `${JSON.stringify(orderedSources)}\n`;

if (checkOnly) {
  const existing = await readFile(outputPath, "utf8");
  if (existing !== output) {
    throw new Error(
      "editors/vscode/scaffold-sources.json is stale; run `bun run --cwd editors/vscode sync:sources`",
    );
  }
} else {
  try {
    await writeFile(temporaryOutputPath, output);
    await rename(temporaryOutputPath, outputPath);
  } catch (error) {
    await rm(temporaryOutputPath, { force: true });
    throw error;
  }
}

async function addSources(root, virtualRoot) {
  for (const file of await schemeFiles(root)) {
    const virtualPath = [virtualRoot, portablePath(relative(root, file))]
      .filter(Boolean)
      .join("/");
    sources[virtualPath] = await readFile(file, "utf8");
  }
}

async function schemeFiles(root) {
  const files = [];
  await collectSchemeFiles(root, files);
  files.sort();
  return files;
}

async function collectSchemeFiles(dir, output) {
  for (const entry of await readdir(dir, { withFileTypes: true })) {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) {
      await collectSchemeFiles(path, output);
    } else if (entry.isFile() && entry.name.endsWith(".scm")) {
      output.push(path);
    }
  }
}

function portablePath(path) {
  return path.split(sep).join("/");
}

function scriptArgs() {
  return parseArgs({
    options: {
      check: { type: "boolean", default: false },
    },
    strict: true,
  }).values;
}
