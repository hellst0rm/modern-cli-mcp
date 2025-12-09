# Dockerfile
# Multi-stage build for Modern CLI MCP using Nix

FROM nixos/nix:latest AS builder

# Enable flakes
RUN echo "experimental-features = nix-command flakes" >> /etc/nix/nix.conf

WORKDIR /build

# Copy source
COPY . .

# Build with Nix and copy the closure to a predictable location
RUN nix build .#full --no-link && \
    mkdir -p /output && \
    cp -rL $(nix build .#full --print-out-paths) /output/app

# Runtime image
FROM nixos/nix:latest

# Enable flakes
RUN echo "experimental-features = nix-command flakes" >> /etc/nix/nix.conf

# Copy the built package (dereferenced, so all files included)
COPY --from=builder /output/app /app

# Set PATH to include the tools
ENV PATH="/app/bin:${PATH}"

ENTRYPOINT ["/app/bin/modern-cli-mcp"]
