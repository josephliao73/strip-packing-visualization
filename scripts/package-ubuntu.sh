#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_DIR="$ROOT_DIR/packing_interface"
DIST_DIR="$ROOT_DIR/dist/ubuntu"
BUNDLE_DIR="$DIST_DIR/packing-app"
BIN_DIR="$BUNDLE_DIR/bin"
SRC_DIR="$BUNDLE_DIR/src"

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Missing required command: $1" >&2
    exit 1
  fi
}

require_cmd cargo
require_cmd python3

echo "Building release binary..."
cargo build --manifest-path "$APP_DIR/Cargo.toml" --release

echo "Assembling Ubuntu bundle at $BUNDLE_DIR"
rm -rf "$BUNDLE_DIR"
mkdir -p "$BIN_DIR" "$SRC_DIR"

cp "$APP_DIR/target/release/packing_interface" "$BIN_DIR/packing_interface"
cp -R "$APP_DIR/src/algorithm_templates" "$SRC_DIR/"
cp -R "$APP_DIR/src/runner_utils" "$SRC_DIR/"
cp -R "$APP_DIR/src/runner_lib" "$SRC_DIR/"
cp "$APP_DIR/requirements.txt" "$BUNDLE_DIR/requirements.txt"

cat > "$BUNDLE_DIR/packing-app" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

BUNDLE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VENV_DIR="$BUNDLE_DIR/.venv"
CPP_MIN_MAJOR=8

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
    echo "Run ./setup-python.sh inside the bundle to install the bundled Python dependencies." >&2
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

if [ ${#RUNTIME_ARGS[@]} -eq 0 ]; then
  echo "No supported Python or C++ runtime detected. The app will start with both language options hidden." >&2
fi

exec "$BUNDLE_DIR/bin/packing_interface" "${RUNTIME_ARGS[@]}"
EOF

cat > "$BUNDLE_DIR/setup-python.sh" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

BUNDLE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VENV_DIR="$BUNDLE_DIR/.venv"

if ! command -v python3 >/dev/null 2>&1; then
  echo "Missing required command: python3" >&2
  exit 1
fi

if [ ! -d "$VENV_DIR" ]; then
  python3 -m venv "$VENV_DIR"
fi

"$VENV_DIR/bin/python3" -m pip install --upgrade pip
"$VENV_DIR/bin/python3" -m pip install -r "$BUNDLE_DIR/requirements.txt"

echo "Bundled Python runtime is ready at $VENV_DIR"
EOF

cat > "$BUNDLE_DIR/README-ubuntu.txt" <<'EOF'
Packing App Ubuntu Bundle
=========================

Contents:
- packing-app: launcher that auto-detects working Python/C++ runtimes
- setup-python.sh: creates a bundle-local .venv and installs numpy/scipy
- bin/packing_interface: compiled Rust GUI binary
- src/: bundled templates and runner helper files

Host requirements:
- Ubuntu desktop session with GUI support
- g++ if you want C++ algorithms enabled in the app
- python3 if you want to create the bundle-local .venv

Recommended first run:
1. ./setup-python.sh
2. ./packing-app

If g++ is installed and supports C++17, the app will also enable C++ templates.
EOF

chmod +x "$BUNDLE_DIR/packing-app" "$BUNDLE_DIR/setup-python.sh"

echo "Ubuntu bundle created:"
echo "  $BUNDLE_DIR"
echo
echo "Run it with:"
echo "  cd \"$BUNDLE_DIR\""
echo "  ./setup-python.sh   # optional, enables Python templates in the bundle"
echo "  ./packing-app"
