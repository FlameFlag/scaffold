{
  lib,
  stdenv,
  bun,
  fetchCrate,
  nodejs_24,
  rustPlatform,
  rustc,
  vscode-utils,
}:

let
  sourceRoot = ./..;
  version = (builtins.fromTOML (builtins.readFile (sourceRoot + "/Cargo.toml"))).package.version;
  fs = lib.fileset;

  src = fs.toSource {
    root = sourceRoot;
    fileset = fs.unions [
      ../Cargo.lock
      ../Cargo.toml
      ../LICENSE
      ../bun.lock
      ../crates
      ../examples
      ../package.json
      ../shared
      ../src

      ../site/index.html
      ../site/package.json
      ../site/scripts
      ../site/src
      ../site/svelte.config.js
      ../site/tsconfig.json
      ../site/vite.config.ts

      ../editors/vscode/.vscodeignore
      ../editors/vscode/LICENSE
      ../editors/vscode/language-configuration.json
      ../editors/vscode/media
      ../editors/vscode/package.json
      ../editors/vscode/scripts
      ../editors/vscode/snippets
      ../editors/vscode/src
      ../editors/vscode/syntaxes
      ../editors/vscode/tsconfig.json
    ];
  };

  nodejs = nodejs_24;

  meta = {
    homepage = "https://github.com/flameflag/scaffold";
    license = lib.licenses.mit;
    maintainers = with lib.maintainers; [ FlameFlag ];
    platforms = lib.platforms.unix;
    sourceProvenance = with lib.sourceTypes; [ fromSource ];
  };

  cargoLock = {
    lockFile = ../Cargo.lock;
  };

  workspaceBunDeps = stdenv.mkDerivation {
    pname = "scaffold-bun-deps";
    inherit version src;

    nativeBuildInputs = [ bun ];

    outputHashMode = "recursive";
    outputHashAlgo = "sha256";
    outputHash = "sha256-NhPebmVYnIdF1i+9wal/bx4LpZLBXjp+teeLkdM6Y1c=";
    dontFixup = true;

    buildPhase = ''
      runHook preBuild
      bun install --frozen-lockfile --ignore-scripts
      runHook postBuild
    '';

    installPhase = ''
      runHook preInstall
      mkdir -p "$out"
      cp -RL node_modules "$out/node_modules"
      runHook postInstall
    '';
  };

  installBunDeps = ''
    cp -R ${workspaceBunDeps}/node_modules node_modules
    chmod -R u+w node_modules
    rm -rf node_modules/.bin
    mkdir -p node_modules/.bin
    ln -s ../vite/bin/vite.js node_modules/.bin/vite
    ln -s ../typescript/bin/tsc node_modules/.bin/tsc
    ln -s ../esbuild/bin/esbuild node_modules/.bin/esbuild
    ln -s ../svelte-check/bin/svelte-check node_modules/.bin/svelte-check
  '';

  wasm-bindgen-cli = rustPlatform.buildRustPackage (finalAttrs: {
    pname = "wasm-bindgen-cli";
    version = "0.2.123";

    src = fetchCrate {
      inherit (finalAttrs) pname version;
      hash = "sha256-ymeAEYsr7OnupWYJWjSeVGvq3+s+zxSNkODbzY62rYs=";
    };

    cargoHash = "sha256-d7x6gtx5OqEE4MyT6yjYn/qtgjx7GroTpXJewnBV2dU=";

    # Upstream tests are intended to run from the wasm-bindgen monorepo.
    doCheck = false;

    meta = {
      homepage = "https://wasm-bindgen.github.io/wasm-bindgen/";
      license = with lib.licenses; [
        asl20
        mit
      ];
      maintainers = with lib.maintainers; [ FlameFlag ];
      platforms = lib.platforms.unix;
      description = "Facilitating high-level interactions between wasm modules and JavaScript";
      mainProgram = "wasm-bindgen";
    };
  });

  scaffold = rustPlatform.buildRustPackage {
    pname = "scaffold";
    inherit version src cargoLock;

    cargoBuildFlags = [ "--package=scaffold" ];
    doCheck = false;

    meta = meta // {
      description = "Command-line tools for Scaffold Scheme catalogs and extensions";
      mainProgram = "scaffold";
    };
  };

  scaffold-wasm = rustPlatform.buildRustPackage {
    pname = "scaffold-wasm";
    inherit version src cargoLock;

    nativeBuildInputs = [
      wasm-bindgen-cli
      rustc.llvmPackages.lld
    ];

    env.CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER = "wasm-ld";
    doCheck = false;

    buildPhase = ''
      runHook preBuild

      cargo build \
        --offline \
        --package scaffold-wasm \
        --release \
        --target wasm32-unknown-unknown

      runHook postBuild
    '';

    installPhase = ''
      runHook preInstall

      mkdir -p "$out/wasm"
      wasm-bindgen \
        target/wasm32-unknown-unknown/release/scaffold_wasm.wasm \
        --target web \
        --remove-name-section \
        --remove-producers-section \
        --out-dir "$out/wasm" \
        --out-name scaffold_wasm

      runHook postInstall
    '';

    meta = meta // {
      description = "WebAssembly bindings for Scaffold editor integrations";
    };
  };

  docs-site = stdenv.mkDerivation {
    pname = "scaffold-docs-site";
    inherit version src;

    sourceRoot = "source";
    doCheck = false;

    nativeBuildInputs = [
      bun
      scaffold
    ];

    NODE_OPTIONS = "--disable-warning=DEP0205";

    buildPhase = ''
      runHook preBuild

      ${installBunDeps}

      chmod -R u+w editors/vscode
      bun editors/vscode/scripts/sync-sources.mjs

      rm -rf editors/vscode/wasm
      cp -R ${scaffold-wasm}/wasm editors/vscode/wasm

      mkdir -p site/.generated
      ${lib.getExe scaffold} docs --output site/.generated/reference.json

      bun run --cwd site static-reference
      bun site/scripts/check-reference.mjs
      bun run --cwd site typecheck
      bun run --cwd site vite build

      runHook postBuild
    '';

    installPhase = ''
      runHook preInstall

      mkdir -p "$out/share/scaffold-docs-site"
      cp -R site/dist/. "$out/share/scaffold-docs-site/"

      runHook postInstall
    '';

    meta = meta // {
      description = "Static documentation site for Scaffold Scheme";
    };
  };

  vscode-extension-vsix = stdenv.mkDerivation {
    name = "scaffold-scheme-${version}.vsix";
    pname = "scaffold-scheme-vscode-vsix";
    inherit version src;

    sourceRoot = "source";
    doCheck = false;

    nativeBuildInputs = [
      bun
      nodejs
    ];

    PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD = "1";

    buildPhase = ''
      runHook preBuild

      ${installBunDeps}

      cd editors/vscode

      bun run sync:sources

      rm -rf wasm
      cp -R ${scaffold-wasm}/wasm wasm

      bun run compile

      # The vsce CLI always runs vscode:prepublish, which rebuilds the WASM
      # with Cargo. Nix already supplied the prepared WASM above.
      bun scripts/package-vsix.mjs --out "$out"
      bun scripts/check-vsix-package.mjs --package "$out"

      runHook postBuild
    '';

    installPhase = ''
      runHook preInstall
      runHook postInstall
    '';

    meta = meta // {
      description = "VSIX package for Scaffold Scheme VS Code support";
    };
  };

  vscode-extension = vscode-utils.buildVscodeExtension {
    pname = "scaffold-scheme";
    inherit version;
    doCheck = false;

    src = vscode-extension-vsix;
    vscodeExtPublisher = "scaffold";
    vscodeExtName = "scaffold-scheme";
    vscodeExtUniqueId = "scaffold.scaffold-scheme";

    meta = meta // {
      description = "VS Code language support for Scaffold Scheme catalogs and extensions";
    };
  };
in
{
  inherit
    docs-site
    scaffold
    scaffold-wasm
    vscode-extension
    vscode-extension-vsix
    wasm-bindgen-cli
    ;

  default = scaffold;
}
