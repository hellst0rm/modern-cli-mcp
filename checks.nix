# checks.nix
{ pkgs, self }:
let
  src = self;

  # Check formatting
  formatCheck = pkgs.runCommand "format-check" { } ''
    cd ${src}
    ${pkgs.fd}/bin/fd -e nix -x ${pkgs.nixfmt-rfc-style}/bin/nixfmt --check
    touch $out
  '';

  # Check for dead code / unused imports
  deadnixCheck = pkgs.runCommand "deadnix-check" { } ''
    cd ${src}
    ${pkgs.fd}/bin/fd -e nix -0 | xargs -0 ${pkgs.deadnix}/bin/deadnix --fail
    touch $out
  '';

  # Check for anti-patterns with statix
  statixCheck = pkgs.runCommand "statix-check" { } ''
    cd ${src}
    ${pkgs.statix}/bin/statix check .
    touch $out
  '';

  # Check Rust formatting
  rustfmtCheck =
    pkgs.runCommand "rustfmt-check"
      {
        nativeBuildInputs = [
          pkgs.cargo
          pkgs.rustfmt
        ];
      }
      ''
        cd ${src}
        cargo fmt --check
        touch $out
      '';

  # Check if system has flakes enabled (Determinate Nix or standard Nix with flakes)
  flakeSystemCheck = pkgs.writeShellScriptBin "flake-system-check" ''
    set -e

    echo "Checking Nix flake support..."

    # Check if nix command is available
    if ! command -v nix &> /dev/null; then
      echo "❌ Error: nix command not found. Please install Nix."
      exit 1
    fi

    # Check Nix version
    NIX_VERSION=$(nix --version 2>/dev/null | head -n1 || echo "unknown")
    echo "✓ Nix installed: $NIX_VERSION"

    # Check if Determinate Nix is installed
    if command -v determinate &> /dev/null; then
      DETERMINATE_VERSION=$(determinate --version 2>/dev/null || echo "unknown")
      echo "✓ Determinate Nix detected: $DETERMINATE_VERSION"
      echo "✓ Flakes are enabled by default in Determinate Nix"
      exit 0
    fi

    # Check if flakes are enabled in standard Nix
    if nix flake metadata --json nixpkgs &> /dev/null 2>&1; then
      echo "✓ Flakes are enabled"
      exit 0
    else
      echo "⚠️  Flakes do not appear to be enabled"
      echo ""
      echo "To enable flakes, add to ~/.config/nix/nix.conf:"
      echo "  experimental-features = nix-command flakes"
      echo ""
      echo "Or install Determinate Nix for automatic flake support:"
      echo "  curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install"
      exit 1
    fi
  '';
in
{
  inherit
    formatCheck
    deadnixCheck
    statixCheck
    rustfmtCheck
    flakeSystemCheck
    ;

  # All checks combined
  all =
    pkgs.runCommand "all-checks"
      {
        buildInputs = [
          formatCheck
          deadnixCheck
          statixCheck
          rustfmtCheck
        ];
      }
      ''
        echo "All checks passed!"
        touch $out
      '';
}
