#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

log() { printf '\n==> %s\n' "$1"; }

if ! command -v cargo >/dev/null 2>&1; then
  echo "ERROR: cargo não encontrado. Instale Rust via rustup." >&2
  exit 1
fi

if ! command -v rustc >/dev/null 2>&1; then
  echo "ERROR: rustc não encontrado." >&2
  exit 1
fi

log "Toolchain"
rustc --version
cargo --version

log "Docker"
if ! command -v docker >/dev/null 2>&1; then
  echo "ERROR: Docker não encontrado; a suíte completa usa Testcontainers/PostgreSQL." >&2
  exit 1
fi
docker info >/dev/null

actions=(
  "cargo fmt --all -- --check"
  "cargo check --workspace --all-targets --all-features"
  "cargo build --workspace --all-targets --all-features"
  "cargo test --workspace --all-targets --all-features --no-run"
  "cargo test --workspace --all-targets"
  "cargo test --workspace --all-targets --all-features -- --test-threads=1"
  "cargo test --workspace --doc --all-features"
  "cargo clippy --workspace --all-targets --all-features -- -D warnings"
  "cargo doc --workspace --no-deps --all-features"
)

for cmd in "${actions[@]}"; do
  log "$cmd"
  eval "$cmd"
done

log "SUITE COMPLETA: OK"
