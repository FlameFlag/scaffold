{
  description = "Scaffold CLI, docs, WASM, and editor packages";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    inputs:
    let
      systems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-linux"
      ];

      forAllSystems = inputs.nixpkgs.lib.genAttrs systems;

      pkgsFor =
        system:
        import inputs.nixpkgs {
          inherit system;
        };
    in
    {
      overlays.default =
        final: _prev:
        let
          scaffoldPackages = final.callPackage ./nix/package.nix { };
        in
        {
          inherit scaffoldPackages;
          scaffold = scaffoldPackages.scaffold;
        };

      packages = forAllSystems (
        system:
        let
          pkgs = pkgsFor system;
        in
        builtins.removeAttrs
          (import ./default.nix {
            inherit pkgs;
          })
          [
            "override"
            "overrideDerivation"
          ]
      );

      apps = forAllSystems (
        system:
        let
          pkgs = pkgsFor system;
          scaffold = inputs.self.packages.${system}.scaffold;
        in
        {
          default = inputs.self.apps.${system}.scaffold;

          scaffold = {
            type = "app";
            program = pkgs.lib.getExe scaffold;
            meta.description = scaffold.meta.description;
          };
        }
      );

      devShells = forAllSystems (
        system:
        let
          pkgs = pkgsFor system;
        in
        {
          default = import ./nix/shell.nix {
            inherit pkgs;
            scaffoldPackages = inputs.self.packages.${system};
          };
        }
      );

      nixosModules.default = ./nix/modules/nixos.nix;
      nixosModules.scaffold = inputs.self.nixosModules.default;

      homeManagerModules.default = ./nix/modules/home-manager.nix;
      homeManagerModules.scaffold = inputs.self.homeManagerModules.default;

      darwinModules.default = ./nix/modules/nix-darwin.nix;
      darwinModules.scaffold = inputs.self.darwinModules.default;
    };
}
