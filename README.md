# packing

this is a desktop ui for 2d strip/bin packing experiments.

you can:
- make or import test cases
- run python or c++ packing code from inside the app
- visualize layouts
- generate batches of random cases
- open a child node tab and repack only a selected region
- save custom root templates and custom node templates

## repo layout

- `packing_interface/` rust app built with `iced`
- `packing_interface/src/algorithm_templates/` builtin and custom templates
- `packing_interface/src/runner_lib/packing_lib.py` python helper lib used by templates
- `packing_interface/src/runner_utils/packing_lib.h` c++ helper lib used by templates
- `start.sh` launcher / runtime checker
- `docs/python-library.md` notes for the python helper library

## runtimes

the app only shows languages that actually work on your machine.

python is considered available only if:
- `python3` exists, or `packing_interface/.venv/bin/python3` exists
- it can import `numpy` and `scipy`

c++ is considered available only if:
- `g++` exists
- it is new enough
- it passes a c++17 compile check

## start script

from the repo root:

```bash
./start.sh install
./start.sh install python
./start.sh install cpp
./start.sh start
./start.sh up
```

what they do:

- `./start.sh install`
  installs both python deps and the c++/rust side checks
- `./start.sh install python`
  creates `packing_interface/.venv` if needed, installs `requirements.txt`, and verifies `numpy` / `scipy`
- `./start.sh install cpp`
  verifies `g++` and c++17 support, then runs `cargo fetch`
- `./start.sh start`
  detects which runtimes actually work and starts the app with only those languages enabled
- `./start.sh up`
  runs install first, then start

you can also pass cargo args after `--`:

```bash
./start.sh start -- --release
./start.sh up -- --release
```

## venv note

the script activates the venv only inside the script process.
that means your terminal prompt will not change after running `./start.sh`.

if you want to activate it manually yourself:

```bash
cd packing_interface
source .venv/bin/activate
```

## running without the script

from `packing_interface/`:

```bash
cargo run
```

but if you do that directly, you are skipping the runtime detection that `start.sh` does.

## templates

templates are stored under:

```text
packing_interface/src/algorithm_templates/
  python/
    root/
    node/
  cpp/
    root/
    node/
```

there is also a manifest at:

```text
packing_interface/src/algorithm_templates/manifest.json
```

root templates are for full packings.
node templates are for repacking a selected region.

builtin templates are read only.
you can duplicate them and save your own editable copies.
that now keeps node copies as node templates and root copies as root templates.

current builtin node default:
- `Blank Node`

current builtin node algorithms:
- `Blank Node`
- obstacle-aware `BL`
- obstacle-aware `NFDH`
- obstacle-aware `FFDH`
- obstacle-aware `BFDH`

node tabs only show algorithms that actually have a node implementation.
so root-only things like lp/fractional templates do not appear there.

## how to use the app

### root tabs

use a root tab for normal packing.

you can:
- enter rectangles manually
- import a testcase json
- generate a random single testcase
- generate a random batch of testcases
- run a root algorithm
- save output json

### node tabs

use a node tab for local repacking inside a selected region.

basic flow:
- run a root algorithm or import an output first
- right click rectangles to select them
- create a node tab from a selected region
- choose a node template from the dropdown
- run repacking on just that region

if nothing is selected, the output panel now tells you that and tells you to use right click.

## read only templates

for builtin templates:
- you can click, highlight, and copy
- you cannot type into them
- you cannot delete selected text from them

if you want to modify one, use the duplicate / save-as flow and edit the custom copy instead.

## output behavior

after a run, the bottom output panel shows:
- json output when the run succeeds
- execution errors when it fails
- layout warnings when the algorithm output is invalid but still renderable

that includes things like:
- rectangle intersections
- out of bounds placements
- node repacking obstacle intersections

invalid layouts still render so you can see what the algorithm tried to do.

## notes on node algorithms

`BL` is the most natural fit for obstacle-heavy node regions.

`NFDH`, `FFDH`, and `BFDH` on node tabs now try their normal shelf behavior first, but if obstacles fragment a level too much they fall back to bottom-left style placement instead of just failing immediately.

## requirements

minimum practical requirements:
- rust toolchain
- python 3 if you want python algorithms
- `numpy` and `scipy` if you want python algorithms to run through the app
- `g++` with c++17 support if you want c++ algorithms

## current status

supported in the ui:
- python
- c++

java is not a real runtime path here right now.

## quick start

if you just want to use it:

```bash
./start.sh install
./start.sh start
```
