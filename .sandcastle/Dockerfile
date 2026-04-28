FROM node:24-bookworm

# Install system dependencies used by OpenCode, GitHub CLI, jj, and Rust builds.
RUN apt-get update && apt-get install -y \
  build-essential \
  ca-certificates \
  clang \
  curl \
  gh \
  git \
  jq \
  libssl-dev \
  llvm \
  pkg-config \
  unzip \
  && rm -rf /var/lib/apt/lists/*

ARG JJ_VERSION=0.40.0

RUN set -eux; \
  arch="$(dpkg --print-architecture)"; \
  case "$arch" in \
    amd64) target="x86_64-unknown-linux-musl" ;; \
    arm64) target="aarch64-unknown-linux-musl" ;; \
    *) echo "unsupported architecture: $arch" >&2; exit 1 ;; \
  esac; \
  curl -fsSL "https://github.com/jj-vcs/jj/releases/download/v${JJ_VERSION}/jj-v${JJ_VERSION}-${target}.tar.gz" \
    | tar -xz -C /usr/local/bin ./jj; \
  chmod +x /usr/local/bin/jj

# Rename the base image's "node" user (UID 1000) to "agent" so bind-mounted
# worktrees stay writable from both the host and container.
RUN usermod -d /home/agent -m -l agent node
RUN mkdir -p /home/agent/.cache /home/agent/.local/share /home/agent/.local/state /home/agent/.config \
  && chmod 0777 /home/agent \
  && chmod -R 0777 /home/agent/.cache /home/agent/.local /home/agent/.config

# Install OpenCode CLI as root before switching users.
RUN npm install -g opencode-ai@latest

USER agent

# Install Bun for repo-local Sandcastle runner commands when needed.
RUN curl -fsSL https://bun.sh/install | bash

# Install Rust nightly to match the devenv Rust channel.
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
  | sh -s -- -y --default-toolchain nightly --component clippy,rustfmt,rust-src,llvm-tools
ENV PATH="/home/agent/.bun/bin:/home/agent/.cargo/bin:$PATH"
RUN cargo install cargo-nextest --locked

WORKDIR /home/agent

# Sandcastle bind-mounts the git worktree and sets it as the working directory
# when the sandbox starts.
ENTRYPOINT ["sleep", "infinity"]
