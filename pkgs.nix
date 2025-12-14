# pkgs.nix
{ pkgs }:
{
  rustTools = with pkgs; [
    # Rust toolchain
    rust-bin.stable.latest.default
    rust-analyzer
    pkg-config
    openssl

    # Nix tools
    nixfmt-rfc-style
  ];

  webTools = with pkgs; [
    # Runtime
    bun

    # Node ecosystem (for compatibility)
    nodejs_22

    # Formatting/Linting
    biome
  ];

  cliTools = with pkgs; [
    # Filesystem
    eza
    bat
    fd
    duf
    dust
    rm-improved # rip - safer rm with graveyard

    # Search
    ripgrep
    fzf
    ast-grep

    # Text processing
    sd
    jq
    yq-go
    qsv
    hck

    # Data transformation (AI-optimized)
    gron # JSON→greppable, JSON output
    htmlq # jq for HTML
    pup # CSS selectors for HTML
    miller # Multi-format processor (JSON/CSV/etc)
    dasel # Universal data selector

    # System
    procs
    tokei
    hyperfine

    # Network
    xh
    doggo
    usql
    curlie # curl with better output

    # Web search
    ddgr # DuckDuckGo CLI with JSON output

    # Git forges
    gh # GitHub CLI
    glab # GitLab CLI

    # Containers
    podman # Rootless containers
    dive # Image layer analysis
    skopeo # Registry operations
    crane # Low-level registry tool
    trivy # Security scanner

    # Kubernetes
    kubectl # K8s CLI
    kubernetes-helm # Helm charts
    stern # Multi-pod logs
    kustomize # Manifest building

    # Diff/Git
    delta
    git
    difftastic
    gnupatch # For file_patch tool

    # Testing
    bats

    # Utility
    file

    # Reference
    tealdeer
    grex
    sad
    navi

    # Archives
    ouch

    # Task queue
    pueue
  ];
}
