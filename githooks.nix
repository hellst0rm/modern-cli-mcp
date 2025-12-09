# githooks.nix - Git hooks configuration using cachix/git-hooks.nix
_: {
  # Nix formatting
  nixfmt-rfc-style.enable = true;

  # Nix linting
  deadnix.enable = true;
  statix.enable = true;

  # Rust formatting
  rustfmt.enable = true;

  # Rust linting
  clippy.enable = true;

  # Cargo check
  cargo-check.enable = true;
}
