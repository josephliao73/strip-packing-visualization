# Bin Packing Visualization Interface

Desktop application for creating 2D bin-packing inputs, running custom packing code, and visualizing packing layouts.

## Repository Layout

- `packing_interface/`: Rust desktop app built with `iced`
- `packing_lib/`: small Python helper package metadata
- `algorithms/`: sample inputs, outputs, and example scripts

## What the App Does

The interface supports four main workflows:

1. Create a bin-packing instance manually and export it as normalized JSON.
2. Import an existing input file and edit it in the UI.
3. Run custom algorithm code from the built-in editor and visualize the result.
4. Import an algorithm output JSON directly and inspect or adjust the layout.

It also supports:

- generating random single test cases
- generating batches of random test cases for quick comparisons
- selecting a sub-region of a packing and repacking only that region in a child tab
- dragging rectangles in the visualization and snapping them to nearby rectangles

## Prerequisites

### Required

- Rust toolchain: <https://www.rust-lang.org/tools/install>
- Python 3

### Recommended for algorithm execution

The embedded code runner looks for `packing_interface/.venv/bin/python3` first, then falls back to `python3` on your `PATH`.

If you want the built-in runner to work reliably, create a virtual environment inside `packing_interface/`:

```bash
cd packing_interface
python3 -m venv .venv
. .venv/bin/activate
pip install -r requirements.txt
```

`numpy` and `scipy` are listed in `packing_interface/requirements.txt`.

## Running the App

```bash
cd packing_interface
cargo run
```

To build without running:

```bash
cd packing_interface
cargo build
```

## How to Use the Interface

### 1. Create a packing input manually

Use the left-side input form:

- `Bin Width`: required
- `Rectangle Count (N)`: optional
- `Rectangle Types (K)`: optional
- `Autofill remaining values`: optional

Enter rectangle definitions in the text editor using:

```text
width height quantity
```

Example:

```text
10 20 5
15 15 3
8 25 2
```

Each line means:

- `width`: rectangle width
- `height`: rectangle height
- `quantity`: number of copies of that rectangle type

Then click `Export Algorithm Input` to save a normalized JSON file.

### 2. Import an existing configuration

Click `Import Configuration`.

Supported file types:

- `.txt`
- `.in`
- `.csv`
- `.json`

Behavior:

- text-based files are loaded directly into the rectangle editor
- JSON files are parsed as the app's input format and populate the form fields automatically

### 3. Use autofill

If `Autofill remaining values` is enabled, the app attempts to complete missing values based on the rectangles already entered.

Autofill behavior:

- if `K` is larger than the current number of rectangle types, the app generates new rectangle types
- if `N` is larger than the current total quantity, the app increases quantities across existing types
- autofill does not remove rectangles or types you already entered

Autofill can fail if the requested `N` and `K` constraints are impossible given the current input.

### 4. Run a custom algorithm from the built-in editor

Open the code/editor panel and load or write your algorithm.

Current practical constraint:

- the app UI shows `Python`, `C++`, and `Java`
- execution is currently routed through the Python runner, so Python is the working path to rely on

Before you can run code in the root tab, load a test case:

- use `Single Test Case` -> `Import Test Case`, or
- use `Single Test Case` -> `Generate Random`, or
- use `Multiple Test Cases` -> generate a batch

Then click `Run`.

Successful runs:

- show the output JSON in the bottom panel
- enable `Show Visualization`
- let you save the produced JSON to disk

### 5. Visualize an output JSON directly

Click `Import Output JSON` and choose a file with the expected output schema.

This is useful if:

- you ran your algorithm outside the app
- you want to inspect a previously saved solution
- you want to manually adjust placements in the canvas

### 6. Work with test cases

The editor's bottom panel has these modes:

- `Single Test Case`: import one test case JSON or generate one random case
- `Multiple Test Cases`: generate many random cases and run the same algorithm on all of them
- `Output`: inspect result JSON and summary output

For random test-case generation:

- `Input size` controls the approximate total number of rectangles
- `Unique types` constrains the number of distinct rectangle dimensions

For multiple test cases:

- enter the number of cases
- click `Generate Test Cases`
- click `Run`
- inspect average height and per-case results
- use `Display` on a result to open that output in a visualization tab

### 7. Use the visualization canvas

Once an output is loaded or produced, the visualization panel lets you:

- zoom in and out
- pan around the layout
- animate rectangle appearance
- drag rectangles
- snap rectangles to nearby edges
- save the current output JSON

### 8. Repack a selected region

The app supports a sub-problem workflow:

1. Run or import a packing result.
2. Select a region in the visualization.
3. Create or use a child tab for that region.
4. Write repacking code for only the selected rectangles.
5. Run the repack operation.

The child tab passes:

- the selected rectangles as the new rectangle set
- overlapping non-selected rectangles as obstacles (`non_empty_space`)
- the selected region dimensions as the local bin

This is intended for experimenting with local improvement heuristics.

## Algorithm Interfaces

### Standard packing algorithm

For root-level runs, define a Python class named `Packing` with this method:

```python
class Packing:
    def solve(self, bin_width, rectangles):
        ...
```

Parameters:

- `bin_width`: integer width of the strip/bin
- `rectangles`: list of `[width, height, quantity]`

Return value:

- a valid JSON string matching the output format shown below

The safest pattern is to return `packing_lib.make_output(...)`.

The default editor template uses helper functions from `packing_lib`:

```python
import packing_lib

class Packing:
    def solve(self, bin_width, rectangles):
        items = packing_lib.expand_items(rectangles)
        placements = []
        total_height = 0

        for rect in items:
            w = rect["width"]
            h = rect["height"]
            placements.append([0, total_height, w, h])
            total_height += h

        return packing_lib.make_output(bin_width, total_height, placements)
```

### Repacking algorithm

For region-repacking in child tabs, define a class named `Repacking`:

```python
class Repacking:
    def solve(self, bin_height, bin_width, rectangles, non_empty_space):
        ...
```

Parameters:

- `bin_height`: height of the selected region
- `bin_width`: width of the selected region
- `rectangles`: selected rectangles, expanded as `[width, height, quantity]`
- `non_empty_space`: obstacle rectangles that overlap the selected region

## File Formats

### Rectangle editor text format

```text
10 20 5
15 15 3
8 25 2
```

### Exported algorithm input JSON

```json
{
  "width_of_bin": 100,
  "number_of_rectangles": 10,
  "number_of_types_of_rectangles": 3,
  "autofill_option": false,
  "rectangle_list": [
    { "width": 10, "height": 20, "quantity": 5 },
    { "width": 15, "height": 15, "quantity": 3 },
    { "width": 8, "height": 25, "quantity": 2 }
  ]
}
```

### Algorithm output JSON

```json
{
  "bin_width": 100,
  "total_height": 65.0,
  "placements": [
    { "x": 0.0, "y": 0.0, "width": 10, "height": 20 },
    { "x": 10.0, "y": 0.0, "width": 15, "height": 15 },
    { "x": 25.0, "y": 0.0, "width": 8, "height": 25 }
  ]
}
```

Field meanings:

- `bin_width`: width of the strip/bin
- `total_height`: total used height
- `placements`: placed rectangles
- `x`, `y`: bottom-left rectangle position in bin coordinates
- `width`, `height`: rectangle dimensions

## Example End-to-End Workflow

1. Start the app with `cargo run` inside `packing_interface/`.
2. Enter a bin width and rectangle list, or import a configuration.
3. Export the input JSON if you need to run an external algorithm.
4. In the code editor, load a single test case or generate one randomly.
5. Write a Python `Packing` class and click `Run`.
6. Review the output JSON in the bottom panel.
7. Click `Show Visualization` to inspect the layout.
8. Optionally save the output JSON or repack a selected region in a child tab.

## Sample Files

Useful sample files live in `algorithms/`, including:

- `algorithm_input.json`
- `algorithm_output.json`
- example Python scripts such as `nfdh.py`, `ffdh.py`, and `fspp.py`

## Notes and Limitations

- The root tab starts with a default Python template.
- The embedded runner currently relies on Python execution even though the UI exposes multiple language labels.
- Test-case loading is required before `Run` is enabled in the root tab.
- Region repacking is only available after a visualization exists.
