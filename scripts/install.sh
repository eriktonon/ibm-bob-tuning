#!/usr/bin/env bash
set -euo pipefail

# One-command installer for IBM Bob v2.1.0+ tuning (macOS / Linux)
# Uses native Rust `bob-tuner` if Cargo is available; otherwise falls back to
# the zero-dependency Node.js engine (`scripts/bob-sync.mjs`) with full parity,
# installing a global `bob-tuner` CLI in `~/.local/bin/bob-tuner` in both modes.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
LOCAL_BIN="${HOME}/.local/bin"
CARGO_BIN="${HOME}/.cargo/bin"
SKIP_RUST=0
DRY_RUN=0
SYNC_ARGS=()

LOCAL_BIN_IN_PATH=0
if [[ ":${PATH}:" == *":${LOCAL_BIN}:"* ]]; then
  LOCAL_BIN_IN_PATH=1
fi

if [[ -d "${CARGO_BIN}" ]] && [[ ":${PATH}:" != *":${CARGO_BIN}:"* ]]; then
  export PATH="${CARGO_BIN}:${PATH}"
fi
if [[ -d "${LOCAL_BIN}" ]] && [[ ":${PATH}:" != *":${LOCAL_BIN}:"* ]]; then
  export PATH="${LOCAL_BIN}:${PATH}"
fi

print_usage() {
  cat <<'EOF'
Usage: ./scripts/install.sh [OPTIONS]

Options:
  --workspace <PATH>   Compile .bob/agents/*.md into the given project workspace (repeatable)
  --model <ID>         Set session.model override during sync (e.g. claude-opus-4-6 or default)
  --all-modes          Expose all 23 modes in custom_modes.yaml instead of the 7 curated modes
  --no-turbo           Skip writing Turbo Mode auto-approval rules into settings.json
  --skip-rust          Skip Cargo compilation and install the zero-dependency Node.js CLI wrapper
  --dry-run            Simulate changes without writing files or modifying PATH
  -h, --help           Show this help message
EOF
}

shell_quote() {
  printf "'%s'" "$(printf '%s' "$1" | sed "s/'/'\\\\''/g")"
}

install_binary_atomic() {
  local src="$1"
  local dst="$2"
  mkdir -p "$(dirname "${dst}")"
  # Remove existing destination before copying so macOS Apple Silicon does not
  # retain a stale code-signature cache on the old Mach-O inode.
  rm -f "${dst}"
  cp "${src}" "${dst}"
  chmod 755 "${dst}"
}

install_node_wrapper() {
  local node_bin="$1"
  local target="${LOCAL_BIN}/bob-tuner"
  local tmp="${target}.tmp.$$"
  local quoted_repo
  local quoted_node
  quoted_repo="$(shell_quote "${REPO_ROOT}")"
  quoted_node="$(shell_quote "${node_bin}")"

  mkdir -p "${LOCAL_BIN}"
  cat > "${tmp}" <<EOF
#!/usr/bin/env bash
set -euo pipefail

DEFAULT_REPO_ROOT=${quoted_repo}
FALLBACK_NODE_BIN=${quoted_node}

REPO_ROOT="\${BOB_TUNING_REPO:-\${DEFAULT_REPO_ROOT}}"
if [[ ! -f "\${REPO_ROOT}/scripts/bob-sync.mjs" ]] && [[ -f "\${HOME}/.bob/settings/.repo-root" ]]; then
  REPO_ROOT="\$(tr -d '\r\n' < "\${HOME}/.bob/settings/.repo-root")"
fi

if [[ ! -f "\${REPO_ROOT}/scripts/bob-sync.mjs" ]]; then
  echo "[ERROR] Could not locate ibm-bob-tuning repository. Set BOB_TUNING_REPO or re-run install.sh." >&2
  exit 1
fi

NODE_BIN="\$(command -v node 2>/dev/null || true)"
if [[ -z "\${NODE_BIN}" && -x "\${FALLBACK_NODE_BIN}" ]]; then
  NODE_BIN="\${FALLBACK_NODE_BIN}"
fi
if [[ -z "\${NODE_BIN}" ]]; then
  echo "[ERROR] Node.js not found in PATH. Install Node.js >= 18 or Rust/Cargo and re-run install.sh." >&2
  exit 1
fi

exec "\${NODE_BIN}" "\${REPO_ROOT}/scripts/bob-sync.mjs" "\$@"
EOF
  chmod 755 "${tmp}"
  mv -f "${tmp}" "${target}"
}

detect_shell_rc() {
  local user_shell
  user_shell="$(basename "${SHELL:-}")"
  case "${user_shell}" in
    zsh)
      printf '%s\n' "${HOME}/.zshrc"
      ;;
    bash)
      if [[ "$(uname -s)" == "Darwin" ]]; then
        printf '%s\n' "${HOME}/.bash_profile"
      else
        printf '%s\n' "${HOME}/.bashrc"
      fi
      ;;
    *)
      printf '%s\n' "${HOME}/.profile"
      ;;
  esac
}

persist_shell_path() {
  local include_cargo="$1"
  local rc_file
  rc_file="$(detect_shell_rc)"
  local need_local=0
  local need_cargo=0

  if ! grep -qE '(^|[:"'\''/])\.local/bin([:"'\''/]|$)' "${rc_file}" 2>/dev/null; then
    need_local=1
  fi
  if [[ "${include_cargo}" -eq 1 ]] && ! grep -qE '(^|[:"'\''/])\.cargo/bin([:"'\''/]|$)' "${rc_file}" 2>/dev/null; then
    need_cargo=1
  fi

  if [[ "${need_local}" -eq 0 && "${need_cargo}" -eq 0 ]]; then
    return 0
  fi

  touch "${rc_file}"
  {
    echo ""
    echo "# >>> bob-tuner >>>"
    if [[ "${need_local}" -eq 1 ]]; then
      echo 'export PATH="$HOME/.local/bin:$PATH"'
    fi
    if [[ "${need_cargo}" -eq 1 ]]; then
      echo 'export PATH="$HOME/.cargo/bin:$PATH"'
    fi
    echo "# <<< bob-tuner <<<"
  } >> "${rc_file}"
  echo "[PATH] Persisted CLI PATH in ${rc_file}"
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    -h|--help)
      print_usage
      exit 0
      ;;
    --skip-rust)
      SKIP_RUST=1
      shift
      ;;
    --workspace|--model)
      if [[ $# -lt 2 ]]; then
        echo "[ERROR] Missing value for $1" >&2
        exit 2
      fi
      SYNC_ARGS+=("$1" "$2")
      shift 2
      ;;
    --workspace=*|--model=*)
      SYNC_ARGS+=("$1")
      shift
      ;;
    --dry-run)
      DRY_RUN=1
      SYNC_ARGS+=("$1")
      shift
      ;;
    --all-modes|--no-turbo)
      SYNC_ARGS+=("$1")
      shift
      ;;
    *)
      echo "Unknown option: $1 (Remember to quote workspace paths containing spaces: --workspace \"/path/to/project\")" >&2
      exit 2
      ;;
  esac
done

echo "================================================================================"
echo " IBM BOB v2.1.0+ TUNING INSTALLER (PE-AGENTS-IBM)"
echo " Repo: ${REPO_ROOT}"
echo "================================================================================"

USED_RUST=0
INSTALLED_CLI=""
CLI_RUNTIME=""
SHELL_RC="$(detect_shell_rc)"

if [[ "${SKIP_RUST}" -eq 0 ]] && command -v cargo >/dev/null 2>&1; then
  echo "[1/2] Rust/Cargo detected -> compiling native bob-tuner (release)..."
  if (cd "${REPO_ROOT}" && cargo build --release); then
    BIN="${REPO_ROOT}/target/release/bob-tuner"
    if [[ "${DRY_RUN}" -eq 0 ]] && [[ -x "${BIN}" ]]; then
      install_binary_atomic "${BIN}" "${CARGO_BIN}/bob-tuner"
      install_binary_atomic "${BIN}" "${LOCAL_BIN}/bob-tuner"
      persist_shell_path 1
      INSTALLED_CLI="${LOCAL_BIN}/bob-tuner"
      CLI_RUNTIME="native Rust"
    fi
    echo "[2/2] Running native bob-tuner sync..."
    "${BIN}" --repo "${REPO_ROOT}" sync ${SYNC_ARGS[@]+"${SYNC_ARGS[@]}"}
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
  NODE_BIN="$(command -v node)"
  if [[ "${DRY_RUN}" -eq 0 ]]; then
    install_node_wrapper "${NODE_BIN}"
    persist_shell_path 0
    INSTALLED_CLI="${LOCAL_BIN}/bob-tuner"
    CLI_RUNTIME="Node.js wrapper"
  fi
  echo "[1/2] Running zero-dependency Node engine (scripts/bob-sync.mjs)..."
  "${NODE_BIN}" "${REPO_ROOT}/scripts/bob-sync.mjs" sync ${SYNC_ARGS[@]+"${SYNC_ARGS[@]}"}
  echo "[2/2] Running deterministic validation..."
  "${NODE_BIN}" "${REPO_ROOT}/scripts/bob-sync.mjs" validate
fi

echo ""
echo "[DONE] Next step inside IBM Bob IDE:"
echo "  1. Press Cmd/Ctrl+Shift+P -> 'Developer: Reload Window' (loads all MCP servers)"
echo "  2. Open a new chat (+) — 'PE-AGENTS-IBM Orquestrador' is now default"
NEXT_STEP=3
if [[ -n "${INSTALLED_CLI}" ]]; then
  echo "  ${NEXT_STEP}. Global CLI installed (${CLI_RUNTIME}): ${INSTALLED_CLI}"
  if [[ "${LOCAL_BIN_IN_PATH}" -eq 0 ]]; then
    echo "     Run 'source \"${SHELL_RC}\"' (or open a new terminal), then run:"
  else
    echo "     Ready to use from any folder:"
  fi
  echo "     bob-tuner status"
  echo "     bob-tuner set-model claude-opus-4-6"
  NEXT_STEP=$((NEXT_STEP + 1))
fi
echo "  ${NEXT_STEP}. For any additional project folder, run:"
echo "     ./scripts/install.sh --workspace \"/path/to/your/project\""
