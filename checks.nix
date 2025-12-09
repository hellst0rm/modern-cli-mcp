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
in
{
  inherit
    formatCheck
    deadnixCheck
    statixCheck
    rustfmtCheck
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
