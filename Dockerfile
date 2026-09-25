# Multi-stage Dockerfile for stateless LuaLaTeX CV Writer MCP server
# Stage 1: Build the Rust binary
FROM rust:1.85-slim AS builder

WORKDIR /usr/src/cv-writer
COPY Cargo.toml Cargo.lock ./
# Pre-cache dependencies
RUN mkdir src templates && \
    echo "fn main() {}" > src/main.rs && \
    touch src/lib.rs && \
    touch templates/star_rover.tex.tera && \
    cargo build --release || true

# Copy real sources and templates
COPY src/ src/
COPY templates/ templates/
RUN cargo build --release

# Stage 2: Runtime image with LuaLaTeX + TeXLive + Fonts
FROM debian:bookworm-slim

ENV DEBIAN_FRONTEND=noninteractive

# Install LuaLaTeX, font packages, and tools
RUN apt-get update && apt-get install -y --no-install-recommends \
    texlive-luatex \
    texlive-latex-recommended \
    texlive-latex-extra \
    texlive-fonts-recommended \
    texlive-fonts-extra \
    fonts-firacode \
    fonts-font-awesome \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Install Fira Sans OTF fonts if not included
RUN apt-get update && apt-get install -y --no-install-recommends fonts-firasans || true && rm -rf /var/lib/apt/lists/*

# Create unprivileged runtime user
RUN useradd -m -u 1000 cvagent
USER cvagent
WORKDIR /home/cvagent

# Copy Rust compiled binary
COPY --from=builder /usr/src/cv-writer/target/release/cv-writer-mcp /usr/local/bin/cv-writer-mcp

# Entrypoint runs the MCP server on stdio with single invoke
ENTRYPOINT ["/usr/local/bin/cv-writer-mcp"]
