#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/packing_interface/src" && pwd)"
VENV_DIR="$ROOT_DIR/.venv"

usage() {
  cat <<'EOF'
Usage:
  ./start.sh install
  ./start.sh start [cargo args...]
  ./start.sh up [cargo args...]

Commands:
  install   Create/update the Python venv, install Python deps, and fetch Rust deps.
  start     Start the app with cargo run.
  up        Run install, then start.
EOF
}

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Missing required command: $1" >&2
    exit 1
  fi
}

activate_venv_if_present() {
  if [ -f "$VENV_DIR/bin/activate" ]; then
    # shellcheck disable=SC1091
    source "$VENV_DIR/bin/activate"
  fi
}

install_deps() {
  require_cmd python3
  require_cmd cargo

  cd "$ROOT_DIR"

  if [ ! -d "$VENV_DIR" ]; then
    python3 -m venv "$VENV_DIR"
  fi

  activate_venv_if_present
  python -m pip install --upgrade pip
  python -m pip install -r requirements.txt

  cargo fetch

  echo "Install complete."
}

start_app() {
  require_cmd cargo
  cd "$ROOT_DIR"
  activate_venv_if_present
  cargo run "$@"
}

main() {
  local cmd="${1:-help}"
  case "$cmd" in
    install)
      shift
      install_deps "$@"
      ;;
    start)
      shift
      start_app "$@"
      ;;
    up)
      shift
      install_deps
      start_app "$@"
      ;;
    help|-h|--help)
      usage
      ;;
    *)
      echo "Unknown command: $cmd" >&2
      usage >&2
      exit 1
      ;;
  esac
}

main "$@"
