{
  pkgs ? import <nixpkgs> { },
}:
pkgs.mkShellNoCC {
  name = "p2pchat";
  packages = with pkgs; [
    bacon
    cargo
    clippy
    git
    gcc
    rustc
    rust-analyzer
    openssl
    pkg-config
  ];
}
