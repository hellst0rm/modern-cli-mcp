# modern-cli-mcp/default.nix
# Compatibility shim for non-flake Nix users
# Usage:
#   nix-build
#   nix-shell -p '(import (fetchTarball "https://github.com/hellst0rm/modern-cli-mcp/archive/main.tar.gz") {})'
#   nix-env -if https://github.com/hellst0rm/modern-cli-mcp/archive/main.tar.gz
(import (
  let
    lock = builtins.fromJSON (builtins.readFile ./flake.lock);
  in
  fetchTarball {
    url = "https://github.com/edolstra/flake-compat/archive/${lock.nodes.flake-compat.locked.rev}.tar.gz";
    sha256 = lock.nodes.flake-compat.locked.narHash;
  }
) { src = ./.; }).defaultNix
