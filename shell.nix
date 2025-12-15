# shell.nix
{
  inputs,
  pkgs,
  pre-commit-check,
  ...
}:
let
  inherit (pkgs.stdenv.hostPlatform) system;

  scripts = import ./scripts {
    inherit pkgs;
    inherit (inputs.pog.packages.${system}) pog;
  };

  inherit (import ./pkgs.nix { inherit pkgs; }) cliTools rustTools webTools;

  rustToolchain = pkgs.rust-bin.stable.latest.default;
in
{
  default = inputs.devshell.legacyPackages.${system}.mkShell {
    name = "modern-cli-mcp";

    motd = ''
      ╔═══════════════════════════════════════════════╗
      ║  Modern CLI MCP Development Shell             ║
      ╚═══════════════════════════════════════════════╝

      Quick Commands:
        build         Build the MCP server
        run           Run the MCP server
        test          Run tests
        check         Run cargo check
        fmt           Format code
        menu          Show all commands

      Run 'menu' for complete command list.
    '';

    packages =
      rustTools
      ++ webTools
      ++ cliTools
      ++ [
        pkgs.fh # FlakeHub CLI
      ];

    commands = [
      # Build & Run
      {
        name = "build";
        category = "build";
        help = "Build the MCP server (release mode)";
        command = "cargo build --release";
      }
      {
        name = "run";
        category = "build";
        help = "Run the MCP server";
        command = "cargo run";
      }
      {
        name = "run-release";
        category = "build";
        help = "Run the MCP server (release mode)";
        command = "cargo run --release";
      }

      # Testing
      {
        name = "test";
        category = "test";
        help = "Run all tests";
        command = "cargo test";
      }
      {
        name = "test-verbose";
        category = "test";
        help = "Run tests with output";
        command = "cargo test -- --nocapture";
      }

      # Quality
      {
        name = "check";
        category = "quality";
        help = "Run cargo check";
        command = "cargo check";
      }
      {
        name = "clippy";
        category = "quality";
        help = "Run clippy lints";
        command = "cargo clippy -- -W clippy::pedantic";
      }
      {
        name = "fmt";
        category = "quality";
        help = "Format Rust code";
        command = "cargo fmt";
      }
      {
        name = "fmt-check";
        category = "quality";
        help = "Check Rust formatting";
        command = "cargo fmt --check";
      }
      {
        name = "fmt-nix";
        category = "quality";
        help = "Format Nix files";
        command = "nixfmt .";
      }

      # Nix
      {
        name = "nix-build";
        category = "nix";
        help = "Build with Nix";
        command = "nix build";
      }
      {
        name = "nix-run";
        category = "nix";
        help = "Run via Nix";
        command = "nix run";
      }
      {
        name = "flake-check";
        category = "nix";
        help = "Check flake";
        command = "nix flake check";
      }
      {
        name = "flake-show";
        category = "nix";
        help = "Show flake outputs";
        command = "nix flake show";
      }
      {
        name = "flake-system-check";
        category = "nix";
        help = "Check if system has Determinate Nix or flakes enabled";
        command = "nix run .#flakeSystemCheck";
      }
      {
        name = "update";
        category = "nix";
        help = "Update flake inputs";
        command = "nix flake update";
      }

      # FlakeHub
      {
        name = "fh-search";
        category = "flakehub";
        help = "Search FlakeHub for flakes";
        command = ''fh search "$@"'';
      }
      {
        name = "fh-list";
        category = "flakehub";
        help = "List flake versions on FlakeHub";
        command = ''fh list "$@"'';
      }
      {
        name = "fh-resolve";
        category = "flakehub";
        help = "Resolve FlakeHub reference to URL";
        command = ''fh resolve "$@"'';
      }
      {
        name = "fh-init";
        category = "flakehub";
        help = "Initialize flake.nix with FlakeHub inputs";
        command = "fh init";
      }
      {
        name = "fh-add";
        category = "flakehub";
        help = "Add FlakeHub input to flake.nix";
        command = ''fh add "$@"'';
      }

      # Scripts (pog-based)
      {
        name = "tools";
        category = "scripts";
        help = "List available CLI tools and their versions";
        command = ''
          ${scripts.tools}/bin/tools "$@"
        '';
      }

      # Website
      {
        name = "web-dev";
        category = "website";
        help = "Start website dev server";
        command = "cd website && bun install && bun run dev";
      }
      {
        name = "web-build";
        category = "website";
        help = "Build website for production";
        command = "cd website && bun install && bun run build";
      }
      {
        name = "web-lint";
        category = "website";
        help = "Lint website code";
        command = "cd website && bun run lint";
      }
      {
        name = "web-format";
        category = "website";
        help = "Format website code";
        command = "cd website && bun run format";
      }
    ];

    env = [
      {
        name = "RUST_SRC_PATH";
        value = "${rustToolchain}/lib/rustlib/src/rust/library";
      }
      {
        name = "RUST_BACKTRACE";
        value = "1";
      }
    ];

    devshell.startup.git-hooks.text = pre-commit-check.shellHook;
  };
}
