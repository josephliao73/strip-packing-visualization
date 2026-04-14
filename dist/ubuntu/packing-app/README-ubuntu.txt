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
