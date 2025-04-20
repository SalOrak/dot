# save this as shell.nix
{ pkgs ? import <nixpkgs> {}}:

pkgs.mkShell {
  packages = [ pkgs.pkg-config pkgs.openssl];
}
