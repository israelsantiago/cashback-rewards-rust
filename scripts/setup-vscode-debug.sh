#!/usr/bin/env bash
# Configura o debug do projeto no VS Code (Ubuntu) e corrige a ausência dos
# botões "Run Test | Debug" no rust-analyzer.
#
# Causa principal (verificada no código):
#   os módulos de tests/acceptance/*_it.rs e de repository_integration_tests.rs
#   são declarados sob #[cfg(feature = "postgres-acceptance")]. Sem essa feature
#   habilitada no rust-analyzer, o código é tratado como inativo (aparece
#   acinzentado, "inactive due to #[cfg] directives") e não recebe botões.
#   O atributo do teste (#[test] ou #[tokio::test]) não influencia isso.
#
# O que o script faz:
#   1. Analisa tests/*.rs e lista quais módulos de teste estão atrás de features.
#   2. Procura arquivos em tests/<subpasta>/ que nenhum crate de teste declara
#      via #[path] (nunca compilam). Só reporta; com --wire-orphans corrige.
#   3. Instala/repara rust-analyzer e CodeLLDB (erro "Unable to find the path to
#      the LLDB debug adapter executable").
#   4. Gera .vscode/settings.json (habilita as features), launch.json e
#      extensions.json.
#   5. Valida com cargo: compila e confere se cada módulo gated lista testes.
#
# Uso:
#   scripts/setup-vscode-debug.sh                 # aplica tudo
#   scripts/setup-vscode-debug.sh --check         # só diagnostica (exit 1 se faltar algo)
#   scripts/setup-vscode-debug.sh --start-db      # também sobe o PostgreSQL (docker compose)
#   scripts/setup-vscode-debug.sh --wire-orphans  # declara testes soltos em tests/<grupo>_tests.rs
set -euo pipefail
shopt -s nullglob

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

PKG="cashback-rewards-rust"
LIB_CRATE="cashback_rewards_rust"
CHECK_ONLY=0
START_DB=0
WIRE_ORPHANS=0

log()  { printf '\n==> %s\n' "$1"; }
ok()   { printf '    OK: %s\n' "$1"; }
warn() { printf '    AVISO: %s\n' "$1" >&2; }
die()  { printf 'ERRO: %s\n' "$1" >&2; exit 1; }

usage() { sed -n '2,30p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; }

for arg in "$@"; do
  case "$arg" in
    --check)    CHECK_ONLY=1 ;;
    --start-db) START_DB=1 ;;
    --wire-orphans) WIRE_ORPHANS=1 ;;
    -h|--help)  usage; exit 0 ;;
    *)          echo "Opção desconhecida: $arg" >&2; usage; exit 2 ;;
  esac
done

[[ -f Cargo.toml && -d tests ]] || die "execute dentro do projeto (Cargo.toml e tests/ não encontrados)."

# ---------------------------------------------------------------------------
# 1. Análise: módulos atrás de features e testes soltos
# ---------------------------------------------------------------------------
GATED=()          # "crate<TAB>feature<TAB>caminho" (sem os módulos de support/)
FEATURES=()       # features citadas em #[cfg(feature = "...")] dentro de tests/
ORPHANS=()

analyze_gated() {
  local f line
  for f in tests/*.rs tests/*/main.rs; do
    while IFS= read -r line; do
      [[ -n "$line" ]] && GATED+=("$f"$'\t'"$line")
    done < <(awk '
      /#\[cfg\(feature *= *"/ { match($0, /"[^"]+"/); feat = substr($0, RSTART+1, RLENGTH-2); next }
      /#\[path *= *"/        { match($0, /"[^"]+"/); p = substr($0, RSTART+1, RLENGTH-2)
                               if (feat != "" && p !~ /^support\//) print feat "\t" p; next }
      /^(pub )?mod /          { feat = "" }
    ' "$f")
  done
  while IFS= read -r line; do
    [[ -n "$line" ]] && FEATURES+=("$line")
  done < <(grep -rhoE 'cfg\(feature *= *"[^"]+"' tests 2>/dev/null | sed -E 's/.*"([^"]+)"/\1/' | sort -u)
}

find_orphans() {
  local tops=(tests/*.rs tests/*/main.rs)
  local f rel
  while IFS= read -r f; do
    rel="${f#tests/}"
    case "$rel" in
      support/*) continue ;;                                    # helpers, não são testes
      */*/main.rs) ;;                                           # main.rs aninhado: checado abaixo
      */main.rs) continue ;;                                    # tests/<dir>/main.rs é raiz de crate
    esac
    if ! grep -qF "\"$rel\"" "${tops[@]}" 2>/dev/null; then
      ORPHANS+=("$rel")
    fi
  done < <(find tests -mindepth 2 -type f -name '*.rs' | sort)
}

log "Analisando tests/"
analyze_gated
find_orphans

if ((${#GATED[@]} == 0)); then
  ok "nenhum módulo de teste está atrás de feature"
else
  warn "módulos de teste atrás de feature (sem a feature habilitada no rust-analyzer NÃO ganham botões):"
  for entry in "${GATED[@]}"; do
    IFS=$'\t' read -r crate feat path <<<"$entry"
    printf '      %s  [feature "%s"]  -> %s\n' "$crate" "$feat" "tests/$path" >&2
  done
fi

if ((${#ORPHANS[@]} == 0)); then
  ok "todos os arquivos de tests/*/ estão declarados em algum crate de teste"
else
  for rel in "${ORPHANS[@]}"; do warn "solto (nenhum crate declara): tests/$rel"; done
fi

if ((CHECK_ONLY)); then
  status=0
  if ((${#FEATURES[@]} > 0)) && ! grep -qs 'rust-analyzer.cargo.features' .vscode/settings.json; then
    warn ".vscode/settings.json não habilita 'rust-analyzer.cargo.features' -> botões ausentes nos testes acima"
    status=1
  fi
  if ((${#ORPHANS[@]} > 0 && WIRE_ORPHANS == 0)); then status=1; fi
  ((status == 0)) && ok "diagnóstico limpo"
  exit "$status"
fi

WIRED_STEMS=()

wire_orphans() {
  local rel group stem mod wrapper uses_support
  for rel in "${ORPHANS[@]}"; do
    group="${rel%%/*}"
    stem="$(basename "$rel" .rs)"
    mod="${stem//[^A-Za-z0-9_]/_}"
    wrapper="tests/${group}_tests.rs"
    uses_support=0
    grep -qE '(crate::)?support::' "tests/$rel" && uses_support=1

    if [[ ! -f "$wrapper" ]]; then
      printf '// Gerado por scripts/setup-vscode-debug.sh: declara os testes de tests/%s/.\n' "$group" > "$wrapper"
      ok "criado $wrapper"
    fi

    if ! grep -qF "\"$rel\"" "$wrapper"; then
      printf '\n#[path = "%s"]\nmod %s;\n' "$rel" "$mod" >> "$wrapper"
      ok "declarado $rel em $wrapper"
    fi

    if ((uses_support)) && ! grep -qF '"support/mod.rs"' "$wrapper"; then
      printf '\n#[path = "support/mod.rs"]\n#[allow(dead_code)]\npub mod support;\n' >> "$wrapper"
      ok "adicionado módulo support em $wrapper (o teste usa crate::support)"
    fi
    WIRED_STEMS+=("$mod")
  done
}

if ((${#ORPHANS[@]} > 0)); then
  if ((WIRE_ORPHANS)); then
    log "Declarando os testes soltos"
    wire_orphans
  else
    warn "rode com --wire-orphans para declará-los automaticamente"
  fi
fi

# ---------------------------------------------------------------------------
# 2. Extensões do VS Code (rust-analyzer + CodeLLDB)
# ---------------------------------------------------------------------------
CODE_BIN=""
for c in code codium; do
  if command -v "$c" >/dev/null 2>&1; then CODE_BIN="$c"; break; fi
done

codelldb_adapter_ok() {
  local d
  for d in "$HOME"/.vscode/extensions/vadimcn.vscode-lldb-* \
           "$HOME"/.vscode-insiders/extensions/vadimcn.vscode-lldb-* \
           "$HOME"/.vscode-oss/extensions/vadimcn.vscode-lldb-* \
           "$HOME"/.vscode-server/extensions/vadimcn.vscode-lldb-*; do
    [[ -x "$d/adapter/codelldb" ]] && return 0
  done
  return 1
}

install_codelldb_vsix() {
  command -v curl >/dev/null 2>&1 || { warn "curl não encontrado; instale o VSIX manualmente."; return 1; }
  local arch re url tmp
  case "$(uname -m)" in
    x86_64)        re='codelldb-(linux-x64|x86_64-linux)\.vsix' ;;
    aarch64|arm64) re='codelldb-(linux-arm64|aarch64-linux)\.vsix' ;;
    *) warn "arquitetura $(uname -m) não suportada pelo fallback"; return 1 ;;
  esac
  url="$(curl -fsSL https://api.github.com/repos/vadimcn/codelldb/releases/latest \
         | grep -oE "https://[^\"]+/${re}" | head -1 || true)"
  [[ -n "$url" ]] || { warn "não achei o VSIX em github.com/vadimcn/codelldb/releases (limite da API?)."; return 1; }
  tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' RETURN
  curl -fL -o "$tmp/codelldb.vsix" "$url"
  "$CODE_BIN" --install-extension "$tmp/codelldb.vsix" --force
}

log "Extensões do VS Code"
if [[ -z "$CODE_BIN" ]]; then
  warn "CLI 'code' não encontrada no PATH. No VS Code: Ctrl+Shift+P > 'Shell Command: Install code command in PATH'."
  warn "Instale manualmente: rust-lang.rust-analyzer e vadimcn.vscode-lldb."
else
  if command -v snap >/dev/null 2>&1 && snap list code >/dev/null 2>&1; then
    warn "VS Code instalado via Snap: o sandbox costuma quebrar o debugger. Prefira o .deb oficial (code.visualstudio.com)."
  fi
  if command -v flatpak >/dev/null 2>&1 && flatpak list 2>/dev/null | grep -qi 'com.visualstudio.code'; then
    warn "VS Code instalado via Flatpak: o sandbox costuma quebrar o debugger. Prefira o .deb oficial."
  fi

  "$CODE_BIN" --install-extension rust-lang.rust-analyzer >/dev/null && ok "rust-analyzer"
  "$CODE_BIN" --install-extension tamasfe.even-better-toml >/dev/null && ok "even-better-toml"
  "$CODE_BIN" --install-extension vadimcn.vscode-lldb >/dev/null || true

  if codelldb_adapter_ok; then
    ok "adaptador codelldb presente"
  else
    warn "adaptador codelldb ausente; reinstalando a extensão do zero"
    "$CODE_BIN" --uninstall-extension vadimcn.vscode-lldb >/dev/null 2>&1 || true
    rm -rf "$HOME"/.vscode/extensions/vadimcn.vscode-lldb-*
    "$CODE_BIN" --install-extension vadimcn.vscode-lldb --force >/dev/null || true
    if ! codelldb_adapter_ok; then
      warn "ainda ausente; tentando o VSIX da plataforma"
      install_codelldb_vsix || true
    fi
    if codelldb_adapter_ok; then
      ok "adaptador codelldb instalado"
    else
      warn "não consegui instalar o adaptador. Baixe o VSIX em github.com/vadimcn/codelldb/releases e rode: code --install-extension <arquivo>.vsix"
    fi
  fi
fi

# ---------------------------------------------------------------------------
# 3. Arquivos .vscode (a pasta está no .gitignore)
# ---------------------------------------------------------------------------
log "Gerando .vscode/"
mkdir -p .vscode

backup() {
  [[ -f "$1" ]] && cp "$1" "$1.bak-$(date +%Y%m%d-%H%M%S)"
  return 0
}

write_file() { # $1 = caminho, $2 = conteúdo
  if [[ -f "$1" ]] && [[ "$(cat "$1")" == "$2" ]]; then ok "$1 já está atualizado"; return; fi
  backup "$1"
  printf '%s\n' "$2" > "$1"
  ok "escrito $1"
}

EXTENSIONS_JSON='{
  "recommendations": [
    "rust-lang.rust-analyzer",
    "vadimcn.vscode-lldb",
    "tamasfe.even-better-toml"
  ]
}'
write_file .vscode/extensions.json "$EXTENSIONS_JSON"

if ((${#FEATURES[@]} > 0)); then
  # "all" acompanha o CI (cargo test --all-features) e novas features futuras
  SETTINGS_JSON='{
  "rust-analyzer.debug.engine": "vadimcn.vscode-lldb",
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.runnables.extraEnv": { "RUST_BACKTRACE": "1" }
}'
else
  SETTINGS_JSON='{
  "rust-analyzer.debug.engine": "vadimcn.vscode-lldb",
  "rust-analyzer.runnables.extraEnv": { "RUST_BACKTRACE": "1" }
}'
fi
if [[ ! -f .vscode/settings.json ]]; then
  printf '%s\n' "$SETTINGS_JSON" > .vscode/settings.json
  ok "escrito .vscode/settings.json"
elif command -v python3 >/dev/null 2>&1; then
  if python3 - .vscode/settings.json "$SETTINGS_JSON" <<'PY'
import json, shutil, sys, time
path, wanted = sys.argv[1], json.loads(sys.argv[2])
with open(path, encoding="utf-8") as fh:
    current = json.load(fh)
merged = {**current, **wanted}
if merged == current:
    print("    OK: .vscode/settings.json já está atualizado")
    sys.exit(0)
shutil.copy(path, f"{path}.bak-{time.strftime('%Y%m%d-%H%M%S')}")
with open(path, "w", encoding="utf-8") as fh:
    json.dump(merged, fh, indent=2, ensure_ascii=False)
    fh.write("\n")
print("    OK: settings.json mesclado (backup criado)")
PY
  then
    :
  else
    warn "settings.json tem comentários/JSON inválido; mescle manualmente:"
    printf '%s\n' "$SETTINGS_JSON" >&2
  fi
else
  warn "python3 ausente; mescle manualmente em .vscode/settings.json:"
  printf '%s\n' "$SETTINGS_JSON" >&2
fi

# Alvos de teste de integração = tests/*.rs (+ tests/*/main.rs), já incluindo os recém-declarados
TARGETS=()
for f in tests/*.rs; do TARGETS+=("$(basename "$f" .rs)"); done
for f in tests/*/main.rs; do TARGETS+=("$(basename "$(dirname "$f")")"); done
((${#TARGETS[@]} > 0)) || die "nenhum alvo de teste encontrado em tests/"
TARGET_OPTIONS="$(printf '"%s", ' "${TARGETS[@]}")"
TARGET_OPTIONS="[${TARGET_OPTIONS%, }]"
TARGET_DEFAULT="${TARGETS[0]}"

LAUNCH_JSON="$(cat <<'JSON'
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug: aplicação (@@PKG@@)",
      "cargo": {
        "args": ["build", "--bin=@@PKG@@", "--package=@@PKG@@"],
        "filter": { "name": "@@PKG@@", "kind": "bin" }
      },
      "args": [],
      "cwd": "${workspaceFolder}",
      "envFile": "${workspaceFolder}/.env",
      "env": { "RUST_LOG": "debug", "RUST_BACKTRACE": "1" }
    },
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug: testes unitários (lib)",
      "cargo": {
        "args": ["test", "--no-run", "--lib", "--package=@@PKG@@"],
        "filter": { "name": "@@LIB@@", "kind": "lib" }
      },
      "args": ["${input:testFilter}", "--nocapture", "--test-threads=1"],
      "cwd": "${workspaceFolder}",
      "env": { "RUST_BACKTRACE": "1" }
    },
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug: teste de integração (escolher alvo)",
      "cargo": {
        "args": ["test", "--no-run", "--all-features", "--test=${input:testTarget}", "--package=@@PKG@@"],
        "filter": { "name": "${input:testTarget}", "kind": "test" }
      },
      "args": ["${input:testFilter}", "--nocapture", "--test-threads=1"],
      "cwd": "${workspaceFolder}",
      "env": { "RUST_BACKTRACE": "1" }
    }
  ],
  "inputs": [
    {
      "id": "testTarget",
      "type": "pickString",
      "description": "Alvo de teste (arquivo de tests/*.rs)",
      "options": @@OPTIONS@@,
      "default": "@@DEFAULT@@"
    },
    {
      "id": "testFilter",
      "type": "promptString",
      "description": "Filtro pelo nome do teste (vazio = todos)",
      "default": ""
    }
  ]
}
JSON
)"
LAUNCH_JSON="${LAUNCH_JSON//@@PKG@@/$PKG}"
LAUNCH_JSON="${LAUNCH_JSON//@@LIB@@/$LIB_CRATE}"
LAUNCH_JSON="${LAUNCH_JSON//@@OPTIONS@@/$TARGET_OPTIONS}"
LAUNCH_JSON="${LAUNCH_JSON//@@DEFAULT@@/$TARGET_DEFAULT}"
write_file .vscode/launch.json "$LAUNCH_JSON"

if [[ ! -f .env && -f .env.example ]]; then
  cp .env.example .env
  ok ".env criado a partir de .env.example"
fi

if ((START_DB)); then
  log "PostgreSQL (docker compose)"
  docker compose up -d --wait postgres
  ok "postgres em localhost:5432"
fi

# ---------------------------------------------------------------------------
# 4. Pré-requisitos dos testes com PostgreSQL e validação com cargo
# ---------------------------------------------------------------------------
if ((${#GATED[@]} > 0)); then
  log "Docker (os testes gated usam Testcontainers/PostgreSQL)"
  if ! command -v docker >/dev/null 2>&1; then
    warn "Docker não encontrado: os botões aparecem, mas postgres_context() falhará ao executar."
  elif ! docker info >/dev/null 2>&1; then
    warn "Docker instalado, mas sem acesso. No Ubuntu: sudo usermod -aG docker \$USER e faça logout/login."
  else
    ok "docker acessível"
  fi
fi

log "Validando com cargo"
if ! command -v cargo >/dev/null 2>&1; then
  warn "cargo não encontrado; pulei a validação. Instale Rust via https://rustup.rs"
else
  cargo test --all-features --no-run
  LIST="$(cargo test --all-features -- --list 2>/dev/null || true)"
  checks=()
  for entry in "${GATED[@]}"; do IFS=$'\t' read -r _ _ path <<<"$entry"; checks+=("$path"); done
  for rel in "${ORPHANS[@]}"; do ((WIRE_ORPHANS)) && checks+=("$rel"); done
  for path in "${checks[@]}"; do
    stem="$(basename "$path" .rs)"; mod="${stem//[^A-Za-z0-9_]/_}"
    n="$(grep -cE "^${mod}::.*: test$" <<<"$LIST" || true)"
    if ((n > 0)); then ok "$mod: $n teste(s) descoberto(s) com todas as features"
    else warn "$mod: nenhum teste descoberto (confira o arquivo)"; fi
  done
fi

cat <<'EOF'

==> Pronto. Falta só recarregar o VS Code:
    Ctrl+Shift+P > "rust-analyzer: Restart server"  (ou "Developer: Reload Window")
    Os botões Run Test | Debug devem aparecer também nos arquivos *_it.rs.
    Testes com PostgreSQL sobem um container via Docker (imagem postgres:18-alpine).
EOF
