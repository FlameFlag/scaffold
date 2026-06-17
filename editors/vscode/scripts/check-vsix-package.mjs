import { mkdtemp, rm, stat } from "node:fs/promises";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { join, posix } from "node:path";
import { parseArgs } from "node:util";

import { packageVsix } from "./package-vsix.mjs";

const require = createRequire(import.meta.url);
const { readZip } = require("@vscode/vsce/out/zip");

const requiredPackageFiles = [
  "extension/package.json",
  "extension/language-configuration.json",
  "extension/scaffold-sources.json",
  "extension/out/browser.js",
  "extension/wasm/scaffold_wasm_bg.wasm",
  "extension/syntaxes/scaffold-scheme.tmlanguage.json",
  "extension/snippets/scaffold-scheme.code-snippets",
];

const forbiddenPackagePrefixes = [
  "extension/src/",
  "extension/scripts/",
  "extension/test-fixtures/",
  "extension/web-test/",
  "extension/node_modules/",
];

const { values } = parseArgs({
  options: {
    package: { type: "string" },
  },
  strict: true,
});

if (values.package) {
  await assertPackage(values.package);
  console.log(`VSIX package check passed (${values.package})`);
} else {
  await assertGeneratedPackage();
}

async function assertGeneratedPackage() {
  const tempDir = await mkdtemp(join(tmpdir(), "scaffold-vsix-"));

  try {
    const expectedPath = join(tempDir, "scaffold-test.vsix");
    const { packagePath, files } = await packageVsix(["--out", expectedPath]);

    assertExpectedPackagePath(packagePath, expectedPath);
    await assertPackage(expectedPath);

    console.log(`VSIX package check passed (${files.length} files)`);
  } finally {
    await rm(tempDir, { recursive: true, force: true });
  }
}

function assertExpectedPackagePath(packagePath, expectedPath) {
  if (packagePath !== expectedPath) {
    throw new Error(`Expected VSIX at ${expectedPath}, got ${packagePath}`);
  }
}

async function assertPackage(packagePath) {
  assertNonEmptyPackage(await stat(packagePath), packagePath);
  await assertPackageContents(packagePath);
}

function assertNonEmptyPackage(packageStats, expectedPath) {
  if (!packageStats.isFile() || packageStats.size === 0) {
    throw new Error(`Expected a non-empty VSIX at ${expectedPath}`);
  }
}

async function assertPackageContents(packagePath) {
  const packageEntries = await readZip(packagePath, () => true);
  const packageFiles = new Set(packageEntries.keys());
  const packageFilesLower = new Set(
    [...packageFiles].map((file) => file.toLowerCase()),
  );
  const manifest = packageManifest(packageEntries);

  assertRequiredPackageFiles(packageFiles);
  assertIgnoredFilesExcluded(packageFiles);
  assertPackageEntrypoints(manifest, packageFilesLower);
  assertContributedFiles(manifest, packageFilesLower);
}

function assertRequiredPackageFiles(packageFiles) {
  for (const file of requiredPackageFiles) {
    if (!packageFiles.has(file)) {
      throw new Error(`VSIX is missing ${file}`);
    }
  }
}

function assertIgnoredFilesExcluded(packageFiles) {
  for (const file of packageFiles) {
    assertPackageFileAllowed(file);
  }
}

function assertPackageFileAllowed(file) {
  const forbiddenPrefix = forbiddenPackagePrefixes.find((prefix) =>
    file.startsWith(prefix),
  );
  if (forbiddenPrefix) {
    throw new Error(`VSIX includes ignored file ${file}`);
  }
}

function packageManifest(packageEntries) {
  return JSON.parse(
    packageEntries.get("extension/package.json").toString("utf8"),
  );
}

function assertPackageEntrypoints(packageJson, packageFiles) {
  if (
    packageJson.main !== "./out/browser.js" ||
    packageJson.browser !== "./out/browser.js"
  ) {
    throw new Error("VSIX package manifest does not point at out/browser.js");
  }
  assertManifestPathIncluded(packageFiles, packageJson.main, "main");
  assertManifestPathIncluded(packageFiles, packageJson.browser, "browser");
}

function assertContributedFiles(packageJson, packageFiles) {
  for (const reference of contributedPackageReferences(packageJson)) {
    assertManifestPathIncluded(packageFiles, reference.path, reference.label);
  }
}

function contributedPackageReferences(packageJson) {
  const contributes = packageJson.contributes ?? {};

  return [
    ...manifestPropertyReferences(
      contributes.languages,
      "configuration",
      "language configuration",
    ),
    ...manifestPropertyReferences(contributes.grammars, "path", "grammar"),
    ...manifestPropertyReferences(contributes.snippets, "path", "snippet"),
    ...walkthroughMediaReferences(contributes.walkthroughs),
  ];
}

function manifestPropertyReferences(items = [], property, label) {
  return items
    .map((item) => ({ label, path: item[property] }))
    .filter((reference) => reference.path);
}

function walkthroughMediaReferences(walkthroughs = []) {
  return walkthroughs.flatMap((walkthrough) =>
    manifestPropertyReferences(
      walkthrough.steps?.map((step) => step.media ?? {}) ?? [],
      "markdown",
      "walkthrough media",
    ),
  );
}

function assertManifestPathIncluded(packageFiles, manifestPath, label) {
  if (!manifestPath) {
    return;
  }

  const packagePath = manifestPackagePath(manifestPath);
  if (!packageFiles.has(packagePath.toLowerCase())) {
    throw new Error(`VSIX manifest ${label} path is missing: ${manifestPath}`);
  }
}

function manifestPackagePath(manifestPath) {
  if (posix.isAbsolute(manifestPath)) {
    throw new Error(`VSIX manifest path must be relative: ${manifestPath}`);
  }

  const normalized = posix.normalize(manifestPath.replace(/^\.\//, ""));
  if (
    normalized === "." ||
    normalized === ".." ||
    normalized.startsWith("../")
  ) {
    throw new Error(`VSIX manifest path escapes package root: ${manifestPath}`);
  }

  return `extension/${normalized}`;
}
