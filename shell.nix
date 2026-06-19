{pkgs ? import <nixpkgs> {}}: with pkgs; mkShell {
  packages = [
    rustc
    cargo
    rust-analyzer
    rustfmt
  ];
}
