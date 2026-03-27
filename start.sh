#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/packing_interface" && pwd)"
VENV_DIR="$ROOT_DIR/.venv"
CPP_MIN_MAJOR=8

usage() {
  cat <<'EOF'
Usage:
  ./start.sh install [python] [cpp]
  ./start.sh start [python] [cpp] [-- cargo args...]
  ./start.sh up [python] [cpp] [-- cargo args...]

Commands:
  install   Install selected runtimes and enforce numpy/scipy + g++ C++17 checks.
  start     Start the app and pass the selected runtime flags through to the Rust app.
  up        Run install, then start.

If no runtime target is given to start/up, the script auto-detects available runtimes.
EOF
}

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Missing required command: $1" >&2
    exit 1
  fi
}

require_any_cmd() {
  for cmd in "$@"; do
    if command -v "$cmd" >/dev/null 2>&1; then
      return 0
    fi
  done

  echo "Missing required command: one of [$*]" >&2
  exit 1
}

activate_venv_if_present() {
  if [ -f "$VENV_DIR/bin/activate" ]; then
    # shellcheck disable=SC1090
    source "$VENV_DIR/bin/activate"
  fi
}

python_runtime_cmd() {
  if [ -x "$VENV_DIR/bin/python3" ]; then
    printf '%s\n' "$VENV_DIR/bin/python3"
    return 0
  fi

  if command -v python3 >/dev/null 2>&1; then
    command -v python3
    return 0
  fi

  return 1
}

verify_python_runtime() {
  local py
  py="$(python_runtime_cmd)" || {
    echo "Python runtime unavailable: python3 not found." >&2
    return 1
  }

  if ! "$py" -c "import numpy, scipy" >/dev/null 2>&1; then
    echo "Python runtime unavailable: failed to import numpy/scipy with $py." >&2
    echo "Run ./start.sh install python to set up the Python dependencies." >&2
    return 1
  fi
}

verify_cpp_runtime() {
  if ! command -v g++ >/dev/null 2>&1; then
    echo "C++ runtime unavailable: g++ not found." >&2
    return 1
  fi

  local version major
  version="$(g++ -dumpfullversion -dumpversion 2>/dev/null | head -n 1)"
  major="${version%%.*}"

  if [[ ! "$major" =~ ^[0-9]+$ ]] || [ "$major" -lt "$CPP_MIN_MAJOR" ]; then
    echo "C++ runtime unavailable: g++ $version is too old. Need g++ $CPP_MIN_MAJOR+ with C++17 support." >&2
    return 1
  fi

  local tmp_bin
  tmp_bin="$(mktemp /tmp/packing-cpp-check-XXXXXX)"
  if ! printf 'int main() { return 0; }\n' | g++ -std=c++17 -x c++ - -o "$tmp_bin" >/dev/null 2>&1; then
    rm -f "$tmp_bin"
    echo "C++ runtime unavailable: g++ failed a C++17 compile check." >&2
    return 1
  fi

  rm -f "$tmp_bin"
}

install_python_deps() {
  local py

  require_cmd python3
  require_any_cmd pip pip3
  cd "$ROOT_DIR"

  if [ ! -d "$VENV_DIR" ]; then
    python3 -m venv "$VENV_DIR"
  fi

  activate_venv_if_present
  py="$(python_runtime_cmd)"
  "$py" -m pip install --upgrade pip
  "$py" -m pip install -r requirements.txt
  verify_python_runtime

  echo "Python install complete. Verified numpy/scipy imports."
}

install_cpp_deps() {
  require_cmd cargo
  require_cmd g++
  cd "$ROOT_DIR"
  verify_cpp_runtime
  cargo fetch
  echo "C++/Rust install complete. Verified g++ C++17 support."
}

detect_runtime_args() {
  RUNTIME_ARGS=()

  if verify_python_runtime >/dev/null 2>&1; then
    RUNTIME_ARGS+=(python)
  else
    verify_python_runtime || true
  fi

  if verify_cpp_runtime >/dev/null 2>&1; then
    RUNTIME_ARGS+=(cpp)
  else
    verify_cpp_runtime || true
  fi
}

start_app() {
  require_cmd cargo
  cd "$ROOT_DIR"
  activate_venv_if_present
  cargo run "$@" -- "${RUNTIME_ARGS[@]}"
}

main() {
  local cmd="${1:-help}"
  shift || true

  local do_python=false
  local do_cpp=false
  local saw_target=false
  local cargo_args=()

  while (($#)); do
    case "$1" in
      python)
        do_python=true
        saw_target=true
        shift
        ;;
      cpp)
        do_cpp=true
        saw_target=true
        shift
        ;;
      --)
        shift
        cargo_args=("$@")
        break
        ;;
      *)
        cargo_args+=("$1")
        shift
        ;;
    esac
  done

  case "$cmd" in
    install)
      if ! $saw_target; then
        install_python_deps
        install_cpp_deps
      else
        $do_python && install_python_deps
        $do_cpp && install_cpp_deps
      fi
      ;;
    start)
      if ! $saw_target; then
        detect_runtime_args
      else
        RUNTIME_ARGS=()
        if $do_python; then
          verify_python_runtime
          RUNTIME_ARGS+=(python)
        fi
        if $do_cpp; then
          verify_cpp_runtime
          RUNTIME_ARGS+=(cpp)
        fi
      fi

      if [ ${#RUNTIME_ARGS[@]} -eq 0 ]; then
        echo "No supported Python or C++ runtime detected. The app will start with both language options hidden." >&2
      fi

      start_app "${cargo_args[@]}"
      ;;
    up)
      if ! $saw_target; then
        install_python_deps
        install_cpp_deps
        detect_runtime_args
      else
        RUNTIME_ARGS=()
        if $do_python; then
          install_python_deps
          RUNTIME_ARGS+=(python)
        fi
        if $do_cpp; then
          install_cpp_deps
          RUNTIME_ARGS+=(cpp)
        fi
      fi

      start_app "${cargo_args[@]}"
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
