# Packing

This is a desktop UI for 2D Strip packing visualizations.

<img width="4384" height="2326" alt="image" src="https://github.com/user-attachments/assets/d2ce203c-7787-48fb-b6eb-b3fde868ac53" />

## Quick start

if you just want to use it:

```bash
./start.sh install
./start.sh start
```

Features:
- Make or import test cases
- Run Python or C++ packing algorithms from inside the app
- Visualize layouts
- Generate batches of random cases
- Ppen a child node tab and repack only a selected region
- Save custom root templates and custom node templates

## Repo layout

- `packing_interface/` rust app built with `iced`
- `packing_interface/src/algorithm_templates/` builtin and custom templates
- `packing_interface/src/runner_lib/packing_lib.py` python helper lib used by templates
- `packing_interface/src/runner_utils/packing_lib.h` c++ helper lib used by templates
- `start.sh` Launcher / Runtime checker
- `docs/python-library.md` Docs for the python helper library

## Runtimes

The app only shows languages that actually work on your machine.

Python is considered available only if:
- `python3` exists, or `packing_interface/.venv/bin/python3` exists
- It can import `numpy` and `scipy`

C++ is considered available only if:
- `g++` exists
- It passes a C++17 compile check

## Start Script

From the repo root:

```bash
./start.sh install
./start.sh install python
./start.sh install cpp
./start.sh start
./start.sh up
```

What they do:

- `./start.sh install`
  Installs both Python deps and the C++/Rust side checks
- `./start.sh install python`
  Creates `packing_interface/.venv` if needed, installs `requirements.txt`, and verifies `numpy` / `scipy`
- `./start.sh install cpp`
  Verifies `g++` and c++17 support, then runs `cargo fetch`
- `./start.sh start`
  Detects which runtimes actually work and starts the app with only those languages enabled
- `./start.sh up`
  Runs install first, then start

You can also pass cargo args after `--`:

```bash
./start.sh start -- --release
./start.sh up -- --release
```

## Running without the script

From `packing_interface/`:

```bash
cargo run
```

This skips the runtime detection that `start.sh` does.

## Templates

<img width="670" height="1056" alt="image" src="https://github.com/user-attachments/assets/e49b1f09-6b06-4854-bd12-ac1a05ab0625" />


Templates are stored under:

```text
packing_interface/src/algorithm_templates/
  python/
    root/
    node/
  cpp/
    root/
    node/
```

There is also a manifest at:

```text
packing_interface/src/algorithm_templates/manifest.json
```

Root templates are for full packings.
Node templates are for repacking a selected region.

Builtin templates are read only.
You can duplicate them and save your own editable copies.
That now keeps node copies as node templates and root copies as root templates.

Current builtin node default:
- `Blank Node`

Current builtin node algorithms:
- `Blank Node`
- obstacle-aware `BL`
- obstacle-aware `NFDH`
- obstacle-aware `FFDH`
- obstacle-aware `BFDH`

Node tabs only show algorithms that actually have a node implementation.

## How to use the app

### Root tabs

use a root tab for normal packing.

You can:
- Create input either by entering rectangles manually, generating a random single testcase, generating a random batch of testcases
- Import a testcase json
- Run a root algorithm
- See visual output
- Drag via double left clicking
- Save output json
- Right click to select a region that can be used for local repacking

<img width="2884" height="2300" alt="image" src="https://github.com/user-attachments/assets/bd978beb-59ab-4c08-98d3-ed2eae0fe748" />


### Node Tabs

Use a node tab for local repacking inside a selected region.

Basic flow:
- Run a root algorithm or import an output first
- Right click rectangles to select them
- Create a node tab from a selected region
- Choose a node template from the dropdown
- Run repacking on just that region

<img width="2886" height="2310" alt="image" src="https://github.com/user-attachments/assets/7bfa3e2b-6202-42fa-b24f-12aac56e9340" />


## Read only templates

For builtin templates, they are read only. If you want to modify one, use the duplicate button flow and edit the custom copy instead.

## Output Behavior

After a run, the bottom output panel shows:
- Json output when the run succeeds
- Execution errors when it fails
- Layout warnings when the algorithm output is invalid but still renderable

Warnings are result of:
- rectangle intersections
- out of bounds placements
- node repacking obstacle intersections

Invalid layouts still render so you can see what the algorithm attempted.

## Requirements

Minimum practical requirements:
- Rust toolchain
- Python 3 if you want Python algorithms
- `numpy` and `scipy` if you want Python algorithms to run through the app
- `g++` with C++17 support if you want C++ algorithms


