import { mkdir, readFile, rename, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const siteDir = resolve(scriptDir, "..");
const repoDir = resolve(siteDir, "..");
const outputPath = resolve(siteDir, ".generated/reference.json");
const checkOnly = scriptArgs().check;
const temporaryOutputPath = checkOnly
  ? resolve(tmpdir(), `scaffold-reference-${process.pid}-${Date.now()}.json`)
  : `${outputPath}.${process.pid}.${Date.now()}.tmp`;

try {
  if (!checkOnly) {
    await mkdir(dirname(outputPath), { recursive: true });
  }

  await run([
    "cargo",
    "run",
    "--locked",
    "--manifest-path",
    resolve(repoDir, "Cargo.toml"),
    "--",
    "docs",
    "--format",
    "json",
    "--output",
    temporaryOutputPath,
  ]);

  if (checkOnly) {
    const [current, generated] = await Promise.all([
      readFile(outputPath, "utf8").catch(() => null),
      readFile(temporaryOutputPath, "utf8"),
    ]);

    if (current !== generated) {
      throw new Error(
        `${relativeOutputPath()} is stale; run "bun run --cwd site reference"`,
      );
    }
  } else {
    await rename(temporaryOutputPath, outputPath);
    console.log(`Generated reference at ${outputPath}`);
  }
} finally {
  await rm(temporaryOutputPath, { force: true });
}

async function run(argv) {
  const proc = Bun.spawn(argv, {
    cwd: repoDir,
    stdout: "inherit",
    stderr: "inherit",
  });
  const exitCode = await proc.exited;

  if (exitCode !== 0) {
    throw new Error(`${argv.slice(0, 2).join(" ")} failed with ${exitCode}`);
  }
}

function relativeOutputPath() {
  return ".generated/reference.json";
}

function scriptArgs() {
  return parseArgs({
    options: {
      check: { type: "boolean", default: false },
    },
    strict: true,
  }).values;
}
