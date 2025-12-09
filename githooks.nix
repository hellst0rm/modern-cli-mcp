# githooks.nix - Git hooks configuration using cachix/git-hooks.nix
{ pkgs }:
let
  rustToolchain = pkgs.rust-bin.stable.latest.default;
in
{
  # Nix formatting
  nixfmt-rfc-style.enable = true;

  # Nix linting
  deadnix.enable = true;
  statix.enable = true;

  # Rust formatting - use rust-overlay toolchain
  rustfmt = {
    enable = true;
    packageOverrides.cargo = rustToolchain;
    packageOverrides.rustfmt = rustToolchain;
  };

  # Rust linting - use rust-overlay toolchain
  # clippy also runs cargo check, so we don't need a separate cargo-check hook
  clippy = {
    enable = true;
    packageOverrides.cargo = rustToolchain;
    packageOverrides.clippy = rustToolchain;
  };
}
