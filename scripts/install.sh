#!/usr/bin/env bash
set -euo pipefail

# One-command installer for IBM Bob v2.1.0+ tuning (macOS / Linux)
# Uses native Rust `bob-tuner` if Cargo is available; otherwise falls back to
# the zero-dependency Node.js engine (`scripts/bob-sync.mjs`) with full parity.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
SKIP_RUST=0
SYNC_ARGS=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --skip-rust) SKIP_RUST=1; shift ;;
    --workspace|--model) SYNC_ARGS+=("$1" "$2"); shift 2 ;;
    --all-modes|--no-turbo|--dry-run) SYNC_ARGS+=("$1"); shift ;;
    *) echo "Unknown option: $1" >&2; exit 2 ;;
  esac
done

echo "================================================================================"
echo " IBM BOB v2.1.0+ TUNING INSTALLER (PE-AGENTS-IBM)"
echo " Repo: ${REPO_ROOT}"
echo "================================================================================"

USED_RUST=0
if [[ "${SKIP_RUST}" -eq 0 ]] && command -v cargo >/dev/null 2>&1; then
  echo "[1/2] Rust/Cargo detected -> compiling native bob-tuner (release)..."
  if (cd "${REPO_ROOT}" && cargo build --release); then
    BIN="${REPO_ROOT}/target/release/bob-tuner"
    echo "[2/2] Running native bob-tuner sync..."
    "${BIN}" --repo "${REPO_ROOT}" sync "${SYNC_ARGS[@]}"
    "${BIN}" --repo "${REPO_ROOT}" validate
    USED_RUST=1
  else
    echo "[WARN] Cargo build failed — falling back to zero-deps Node engine." >&2
  fi
fi

if [[ "${USED_RUST}" -eq 0 ]]; then
  if ! command -v node >/dev/null 2>&1; then
    echo "[ERROR] Neither Cargo nor Node.js was found in PATH. Install Node.js >= 18 or Rust/Cargo." >&2
    exit 1
  fi
  echo "[1/2] Running zero-dependency Node engine (scripts/bob-sync.mjs)..."
  node "${REPO_ROOT}/scripts/bob-sync.mjs" sync "${SYNC_ARGS[@]}"
  echo "[2/2] Running deterministic validation..."
  node "${REPO_ROOT}/scripts/bob-sync.mjs" validate
fi

echo ""
echo "[DONE] Next step inside IBM Bob IDE:"
echo "  1. Press Cmd/Ctrl+Shift+P -> 'Developer: Reload Window' (loads all MCP servers)"
echo "  2. Open a new chat (+) — 'PE-AGENTS-IBM Orquestrador' is now default"
echo "  3. For any additional project folder, run:"
echo "     ./scripts/install.sh --workspace /path/to/your/project"
