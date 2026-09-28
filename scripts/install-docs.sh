#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET="${1:-$HOME/Downloads/cashback-rewards-rust}"

if [[ ! -f "$TARGET/Cargo.toml" ]]; then
  echo "ERROR: projeto Rust não encontrado em: $TARGET" >&2
  echo "Uso: $0 /caminho/para/cashback-rewards-rust" >&2
  exit 1
fi

mkdir -p "$TARGET/docs/diagrams" "$TARGET/scripts"
cp "$ROOT/docs/ARCHITECTURE_MIGRATION.md" "$TARGET/docs/ARCHITECTURE_MIGRATION.md"
cp "$ROOT/docs/diagrams/architecture-overview.mmd" "$TARGET/docs/diagrams/architecture-overview.mmd"
cp "$ROOT/docs/diagrams/test-mapping.mmd" "$TARGET/docs/diagrams/test-mapping.mmd"
cp "$ROOT/docs/diagrams/java-rust-architecture-overview.png" "$TARGET/docs/diagrams/java-rust-architecture-overview.png"
cp "$ROOT/docs/diagrams/java-rust-test-mapping.png" "$TARGET/docs/diagrams/java-rust-test-mapping.png"
cp "$ROOT/scripts/verify-rust-suite.sh" "$TARGET/scripts/verify-rust-suite.sh"
chmod +x "$TARGET/scripts/verify-rust-suite.sh"

if ! grep -qF 'docs/ARCHITECTURE_MIGRATION.md' "$TARGET/README.md" 2>/dev/null; then
  cat >> "$TARGET/README.md" <<'EOF'

## Architecture and migration

See [docs/ARCHITECTURE_MIGRATION.md](docs/ARCHITECTURE_MIGRATION.md) for the Java → Rust architecture, technology decisions and Java/Rust test traceability.
EOF
fi

echo "Documentation installed in: $TARGET"
