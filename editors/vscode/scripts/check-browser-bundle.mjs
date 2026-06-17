import { access, readFile, rm } from "node:fs/promises";
import { basename, resolve } from "node:path";
import { runCommand } from "./run-command.mjs";
import { extensionRoot } from "./script-paths.mjs";

const outputDir = resolve(extensionRoot, "out");
const generatedName = `browser.${process.pid}.${Date.now()}.check.js`;
const generatedBundle = resolve(outputDir, generatedName);
const generatedMap = `${generatedBundle}.map`;
const checkedFiles = [
  {
    expected: "browser.js",
    generated: generatedName,
    normalize: normalizeBrowserBundle,
  },
  {
    expected: "browser.js.map",
    generated: `${generatedName}.map`,
    normalize: (value) => value,
  },
];

try {
  if (!(await expectedBundleExists())) {
    console.log("browser bundle not present; skipping freshness check");
    process.exit(0);
  }

  await runCommand(
    [
      "esbuild",
      "src/browser.ts",
      "--bundle",
      "--platform=browser",
      "--format=cjs",
      "--external:vscode",
      "--log-override:empty-import-meta=silent",
      "--sourcemap",
      `--outfile=${generatedBundle}`,
    ],
    { cwd: extensionRoot },
  );

  const stale = [];
  for (const file of checkedFiles) {
    const [expected, generated] = await Promise.all([
      readFile(resolve(outputDir, file.expected), "utf8"),
      readFile(resolve(outputDir, file.generated), "utf8"),
    ]);
    if (file.normalize(expected) !== file.normalize(generated)) {
      stale.push(file.expected);
    }
  }

  if (stale.length > 0) {
    throw new Error(
      `${stale.map((file) => basename(file)).join(", ")} stale; run "bun run --cwd editors/vscode compile"`,
    );
  }
} finally {
  await Promise.all([
    rm(generatedBundle, { force: true }),
    rm(generatedMap, { force: true }),
  ]);
}

function normalizeBrowserBundle(value) {
  return value.replace(/\n?\/\/# sourceMappingURL=.*\n?$/u, "\n");
}

async function expectedBundleExists() {
  return Promise.all(
    checkedFiles.map((file) => access(resolve(outputDir, file.expected))),
  )
    .then(() => true)
    .catch(() => false);
}
