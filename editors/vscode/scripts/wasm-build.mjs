import { execFileSync } from "node:child_process";
import { homedir } from "node:os";
import { resolve } from "node:path";
import { runCommand } from "./run-command.mjs";
import { repoRoot } from "./script-paths.mjs";

const wasmInput = resolve(
  repoRoot,
  "target/wasm32-unknown-unknown/release/scaffold_wasm.wasm",
);

export async function buildScaffoldWasm(options = {}) {
  await runCommand(
    [
      "cargo",
      "build",
      "--locked",
      "--manifest-path",
      resolve(repoRoot, "Cargo.toml"),
      "-p",
      "scaffold-wasm",
      "--target",
      "wasm32-unknown-unknown",
      "--release",
    ],
    {
      ...options,
      env: wasmBuildEnv(options.env),
    },
  );
}

export async function generateWasmBindings(outDir, options = {}) {
  await runCommand(
    [
      "wasm-bindgen",
      wasmInput,
      "--target",
      "web",
      "--remove-name-section",
      "--remove-producers-section",
      "--out-dir",
      outDir,
      "--out-name",
      "scaffold_wasm",
    ],
    options,
  );
}

function wasmBuildEnv(env = process.env) {
  return {
    ...env,
    RUSTFLAGS: normalizedRustFlags(env),
  };
}

function normalizedRustFlags(env) {
  const flags = new Set((env.RUSTFLAGS ?? "").split(/\s+/).filter(Boolean));
  flags.add('--cfg=getrandom_backend="wasm_js"');
  for (const flag of remapPathFlags(env)) {
    flags.add(flag);
  }
  return [...flags].join(" ");
}

function remapPathFlags(env) {
  const home = homedir();
  const cargoHome = env.CARGO_HOME ?? resolve(home, ".cargo");
  const rustupHome = env.RUSTUP_HOME ?? resolve(home, ".rustup");

  return [
    [home, "/home"],
    [rustupHome, "/rustup-home"],
    [cargoHome, "/cargo-home"],
    [repoRoot, "/workspace"],
    [rustSysroot(env), "/rust-sysroot"],
  ]
    .filter(([from]) => from !== "/")
    .map(([from, to]) => `--remap-path-prefix=${from}=${to}`);
}

function rustSysroot(env) {
  return execFileSync("rustc", ["--print", "sysroot"], {
    encoding: "utf8",
    env,
  }).trim();
}
