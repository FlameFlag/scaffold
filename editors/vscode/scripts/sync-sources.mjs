import { readdir, readFile, writeFile } from "node:fs/promises";
import { dirname, join, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

const extensionRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const repoRoot = resolve(extensionRoot, "..", "..");
const dslRoot = join(repoRoot, "crates", "scaffold-dsl", "src");

const sources = {};

await addSources(join(dslRoot, "std"), "src/dsl/std");
await addSources(join(dslRoot, "extensions"), "src/extensions");

const orderedSources = Object.fromEntries(
  Object.entries(sources).sort(([left], [right]) => left.localeCompare(right)),
);

await writeFile(
  join(extensionRoot, "scaffold-sources.json"),
  `${JSON.stringify(orderedSources)}\n`,
);

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
