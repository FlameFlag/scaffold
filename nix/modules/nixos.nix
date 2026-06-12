{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.scaffold;
  scaffoldPackages = pkgs.callPackage ../package.nix { };
in
{
  options.programs.scaffold = {
    enable = lib.mkEnableOption "Scaffold CLI tools";

    package = lib.mkOption {
      type = lib.types.package;
      default = scaffoldPackages.scaffold;
      defaultText = lib.literalExpression "pkgs.callPackage <scaffold>/nix/package.nix { }.scaffold";
      description = "Scaffold CLI package to install.";
    };

    enableVscodeExtension = lib.mkEnableOption "the Scaffold Scheme VS Code extension";

    vscodeExtensionPackage = lib.mkOption {
      type = lib.types.package;
      default = scaffoldPackages.vscode-extension;
      defaultText = lib.literalExpression "pkgs.callPackage <scaffold>/nix/package.nix { }.vscode-extension";
      description = "Scaffold Scheme VS Code extension package to install.";
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [
      cfg.package
    ]
    ++ lib.optional cfg.enableVscodeExtension cfg.vscodeExtensionPackage;
  };
}
