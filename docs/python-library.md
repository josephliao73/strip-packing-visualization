# Python Library Reference

This project exposes a small Python helper module as `packing_lib` for user-written packing algorithms.

The module lives at `src/runner_lib/packing_lib.py` and is injected into the Python runner used by the app. In user code, you can import it directly:

```python
import packing_lib
```

## Runtime Contract

Your Python algorithm must define one of these classes:

```python
class Packing:
    def solve(self, bin_width, rectangles):
        ...
```

or

```python
class Repacking:
    def solve(self, bin_height, bin_width, rectangles, non_empty_space):
        ...
```

### `Packing.solve(self, bin_width, rectangles)`

Arguments:
- `bin_width: int`
- `rectangles: list[list[int, int, int]]`

Each rectangle entry is:
- `[width, height, quantity]`

Return value:
- A JSON string created with `packing_lib.make_output(...)`

### `Repacking.solve(self, bin_height, bin_width, rectangles, non_empty_space)`

Arguments:
- `bin_height: int`
- `bin_width: int`
- `rectangles: list[list[int, int, int]]`
- `non_empty_space: list[list[int, int]]`

`non_empty_space` describes occupied or blocked cells/regions passed into the repacking runner.

Return value:
- A JSON string created with `packing_lib.make_output(...)`

## Output Format

Use `packing_lib.make_output` to return the final layout:

```python
return packing_lib.make_output(bin_width, total_height, placements)
```

Where:
- `bin_width: int`
- `total_height: int | float`
- `placements: list[tuple[x, y, width, height]]`

The produced JSON has this structure:

```json
{
  "bin_width": 10,
  "total_height": 25,
  "placements": [
    {"x": 0, "y": 0, "width": 4, "height": 5},
    {"x": 4, "y": 0, "width": 6, "height": 5}
  ]
}
```

## API Reference

### `make_output(bin_width, total_height, placements)`

Builds the JSON string expected by the app.

Parameters:
- `bin_width`: output bin width
- `total_height`: total occupied strip height
- `placements`: iterable of `(x, y, width, height)` tuples

Returns:
- `str`

### `expand_items(rectangle_list)`

Expands compressed rectangle types into individual items.

Input:
- `[[w, h, q], ...]`

Output:
- `[{"width": w, "height": h}, ...]`

Example:

```python
packing_lib.expand_items([[4, 2, 3], [1, 1, 2]])
# => [
#   {"width": 4, "height": 2},
#   {"width": 4, "height": 2},
#   {"width": 4, "height": 2},
#   {"width": 1, "height": 1},
#   {"width": 1, "height": 1},
# ]
```

### `sort_by_height(items, descending=True)`

Sorts expanded item dicts by `height`.

Expected item format:

```python
{"width": 4, "height": 2}
```

Returns:
- A new sorted list

### `sort_by_width(items, descending=True)`

Sorts expanded item dicts by `width`.

Returns:
- A new sorted list

### `sort_by_area(items, descending=True)`

Sorts expanded item dicts by `width * height`.

Returns:
- A new sorted list

### `sort_by_quantity(rectangle_list, descending=True)`

Sorts rectangle type dicts by `quantity`.

Expected item format:

```python
{"width": 4, "height": 2, "quantity": 3}
```

Returns:
- A new sorted list

Note:
- This helper expects dictionary entries with a `quantity` key, not `[w, h, q]` tuples.

### `dedup_rectangles(rectangle_list)`

Merges duplicate rectangle types by summing their quantities.

Accepted input formats:
- `[[w, h, q], ...]`
- `[{"width": w, "height": h, "quantity": q}, ...]`

Returns:
- The same general shape as the input

Example:

```python
packing_lib.dedup_rectangles([[2, 3, 1], [2, 3, 4], [1, 1, 2]])
# => [[2, 3, 5], [1, 1, 2]]
```

### `get_type_keys(rectangle_list)`

Normalizes a rectangle list into unique type keys and total quantities.

Accepted input formats:
- `[[w, h, q], ...]`
- `[{"width": w, "height": h, "quantity": q}, ...]`

Returns:
- `(type_keys, quantities)`

Where:
- `type_keys` is `list[tuple[int, int]]`
- `quantities` is a parallel `list[int]`

Example:

```python
type_keys, quantities = packing_lib.get_type_keys([
    [2, 3, 1],
    [2, 3, 4],
    [1, 1, 2],
])
# type_keys  => [(2, 3), (1, 1)]
# quantities => [5, 2]
```

### `get_configurations(type_keys, bin_width)`

Generates all valid one-strip configurations for the given rectangle types under a fixed `bin_width`.

Parameters:
- `type_keys: list[tuple[int, int]]`
- `bin_width: int`

Returns:
- A sorted list of count vectors

Each configuration is a list like:

```python
[c0, c1, c2]
```

Meaning:
- `c0` instances of `type_keys[0]`
- `c1` instances of `type_keys[1]`
- `c2` instances of `type_keys[2]`

The function only enforces width feasibility within a strip. It does not check inventory limits or optimize height.

## Minimal Example

```python
import packing_lib

class Packing:
    def solve(self, bin_width, rectangles):
        items = packing_lib.expand_items(rectangles)
        items = packing_lib.sort_by_height(items)

        placements = []
        y = 0

        for item in items:
            placements.append((0, y, item["width"], item["height"]))
            y += item["height"]

        return packing_lib.make_output(bin_width, y, placements)
```

## Notes and Caveats

- `make_output` returns a JSON string, not a Python dict.
- `sort_by_quantity` expects dict inputs with a `quantity` field.
- `expand_items` drops quantity information because it expands each type into unit items.
- `get_configurations` is combinational and may become expensive with many types or large bin widths.
