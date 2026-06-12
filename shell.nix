{
  pkgs ? import <nixpkgs> { },
}:

let
  scaffoldPackages = import ./default.nix {
    inherit pkgs;
  };
in
import ./nix/shell.nix {
  inherit pkgs scaffoldPackages;
}
