#!/usr/bin/env bash
set -euo pipefail

OWNER="${1:-israelsantiago}"
REPO="${2:-cashback-rewards-rust}"
VISIBILITY="${3:---public}"

command -v gh >/dev/null 2>&1 || {
  echo "GitHub CLI (gh) is required. Authenticate with: gh auth login" >&2
  exit 1
}

git rev-parse --is-inside-work-tree >/dev/null 2>&1 || git init -b main

if ! git rev-parse --verify HEAD >/dev/null 2>&1; then
  git add .
  git commit -m "Initial Rust port of Serenity Dojo cashback rewards"
fi

gh repo create "${OWNER}/${REPO}" "${VISIBILITY}" \
  --description "Rust port of Serenity Dojo cashback-rewards using Axum, SQLx and hexagonal architecture" \
  --source=. \
  --remote=origin \
  --push
