# modern-cli-mcp/flake.nix
{
  description = "MCP server exposing modern CLI tools for AI/LLM agents";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    devshell = {
      url = "github:numtide/devshell";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    pog = {
      url = "github:jpetrucciani/pog";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    git-hooks = {
      url = "github:cachix/git-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
      git-hooks,
      ...
    }@inputs:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        inherit (import ./pkgs.nix { inherit pkgs; }) cliTools;

        # Build the MCP server binary
        mcpServer = pkgs.rustPlatform.buildRustPackage {
          pname = "modern-cli-mcp";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = with pkgs; [ pkg-config ];
          buildInputs = with pkgs; [ openssl ];

          buildType = "release";

          meta = with pkgs.lib; {
            description = "MCP server exposing modern CLI tools";
            homepage = "https://github.com/hellst0rm/modern-cli-mcp";
            license = licenses.mit;
            maintainers = [ ];
            mainProgram = "modern-cli-mcp";
          };
        };

        # Wrapped binary with all tools in PATH
        wrappedMcp = pkgs.symlinkJoin {
          name = "modern-cli-mcp-wrapped";
          paths = [ mcpServer ];
          buildInputs = [ pkgs.makeWrapper ];
          postBuild = ''
            wrapProgram $out/bin/modern-cli-mcp \
              --prefix PATH : ${pkgs.lib.makeBinPath cliTools}
          '';
        };

        # Git hooks
        pre-commit-check = git-hooks.lib.${system}.run {
          src = ./.;
          hooks = import ./githooks.nix { inherit pkgs; };
        };
      in
      {
        packages = {
          default = wrappedMcp;
          full = pkgs.symlinkJoin {
            name = "modern-cli-mcp-full";
            paths = [ wrappedMcp ] ++ cliTools;
          };
          server-only = mcpServer;
        };

        devShells = import ./shell.nix {
          inherit inputs pkgs pre-commit-check;
        };

        checks = import ./checks.nix { inherit pkgs self; };

        formatter = pkgs.nixfmt-rfc-style;

        apps.default = flake-utils.lib.mkApp {
          drv = wrappedMcp;
        };
      }
    );
}
