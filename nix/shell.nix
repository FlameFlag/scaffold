{
  pkgs ? import <nixpkgs> { },
  scaffoldPackages ? pkgs.callPackage ./package.nix { },
}:

pkgs.mkShell {
  packages = [
    pkgs.cargo
    pkgs.clippy
    pkgs.bun
    pkgs.nodejs_24
    pkgs.rust-analyzer
    pkgs.rustc
    pkgs.rustfmt
    scaffoldPackages.wasm-bindgen-cli
    pkgs.rustc.llvmPackages.lld
  ];
}
