# Python Library Reference

This project exposes a Python helper module as `packing_lib` for user-written packing algorithms.

The module lives at `src/runner_lib/packing_lib.py` and is injected into the Python runner. Import it directly:

```python
import packing_lib
```

## Runtime Contract

Your algorithm must define one of these classes:

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

Used for strip packing (unbounded height). The solver calls this at the root node.

Parameters:
- `bin_width: int`
- `rectangles: list[tuple[int, int, int]]` — each entry is `(width, height, quantity)`

Return value: a JSON string produced by `packing_lib.make_output(...)` or `packing_lib.output_from_placements(...)`

### `Repacking.solve(self, bin_height, bin_width, rectangles, non_empty_space)`

Used for repacking into a fixed-size bin that already has occupied regions. The solver calls this at interior tree nodes.

Parameters:
- `bin_height: int`
- `bin_width: int`
- `rectangles: list[tuple[int, int, int]]` — each entry is `(width, height, quantity)`
- `non_empty_space: list[dict]` — each entry is a blocked region with keys `x_1`, `x_2`, `y_1`, `y_2`

Return value: a JSON string produced by `packing_lib.make_output(...)` or `packing_lib.output_from_placements(...)`

## Output Format

Use `make_output` or `output_from_placements` to build the return value.

```python
return packing_lib.make_output(bin_width, total_height, placements)
# or equivalently:
return packing_lib.output_from_placements(bin_width, placements)
```

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

---

## API Reference

### Output Helpers

#### `make_output(bin_width, total_height, placements)`

Builds the JSON string expected by the app.

- `bin_width: int | float`
- `total_height: int | float`
- `placements: iterable[tuple[x, y, width, height]]`

Returns `str`.

#### `output_from_placements(bin_width, placements)`

Computes `total_height` automatically from `placements`, then calls `make_output`.

- `bin_width: int | float`
- `placements: list[tuple[x, y, width, height]]`

Returns `str`. Equivalent to `make_output(bin_width, total_height(placements), placements)`.

#### `total_height(placements)`

Returns the maximum `y + height` across all placements, or `0` if the list is empty.

---

### Item Expansion

#### `expand_items(rectangle_list)`

Expands a compressed rectangle list into one dict per item.

- Input: `[[w, h, q], ...]` or `[{"width": w, "height": h, "quantity": q}, ...]`
- Output: `[{"width": w, "height": h}, ...]` (quantity items per type)

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

#### `expand_type_counts(type_keys, quantities)`

Expands parallel arrays of type keys and quantities into a flat item list. Useful after modifying quantities in-place.

- `type_keys: list[tuple[int, int]]`
- `quantities: list[int]`

Returns `list[dict]` with `"width"` and `"height"` keys.

---

### Sorting

All sort functions return a new list and default to descending order.

#### `sort_by_height(items, descending=True)`

Sorts expanded item dicts (`{"width": ..., "height": ...}`) by `height`.

#### `sort_by_width(items, descending=True)`

Sorts expanded item dicts by `width`.

#### `sort_by_area(items, descending=True)`

Sorts expanded item dicts by `width * height`.

#### `sort_by_quantity(rectangle_list, descending=True)`

Sorts rectangle type dicts by `quantity`. Expects `{"width": ..., "height": ..., "quantity": ...}` entries, not `[w, h, q]` tuples.

---

### Rectangle Type Utilities

#### `dedup_rectangles(rectangle_list)`

Merges duplicate `(width, height)` types by summing their quantities.

- Accepts `[[w, h, q], ...]` or `[{"width": w, "height": h, "quantity": q}, ...]`
- Returns the same format as the input

```python
packing_lib.dedup_rectangles([[2, 3, 1], [2, 3, 4], [1, 1, 2]])
# => [[2, 3, 5], [1, 1, 2]]
```

#### `get_type_keys(rectangle_list)`

Deduplicates a rectangle list and returns parallel arrays of unique types and total quantities.

- Accepts `[[w, h, q], ...]` or `[{"width": w, "height": h, "quantity": q}, ...]`
- Returns `(type_keys, quantities)` where `type_keys: list[tuple[int, int]]` and `quantities: list[int]`

```python
type_keys, quantities = packing_lib.get_type_keys([[2, 3, 1], [2, 3, 4], [1, 1, 2]])
# type_keys  => [(2, 3), (1, 1)]
# quantities => [5, 2]
```

#### `get_type_demands(rectangle_list, type_keys=None)`

Like `get_type_keys`, but also computes the total area demand per type (quantity × height).

Returns `(type_keys, quantities, demands)`.

- `demands[i] = quantities[i] * type_keys[i][1]`
- If `type_keys` is provided, uses that ordering instead of deduplicating from scratch.

---

### Strip Placement (no obstacles)

These functions place items into a strip of fixed width and unbounded height.

#### `stack_vertically(items)`

Places each item in a column at `x=0`, stacking them top-to-bottom.

- `items: list[dict]`

Returns `list[tuple[x, y, width, height]]`.

#### `place_items_nfdh(items, bin_width, start_y=0.0)`

Places items using the **Next Fit Decreasing Height** shelf algorithm. Items should be sorted by height descending before calling (the function sorts them internally).

- `items: list[dict]`
- `bin_width: int | float`
- `start_y: float` — y-offset for the first shelf

Returns `list[tuple[x, y, width, height]]`.

#### `place_items_ffdh(items, bin_width, start_y=0.0)`

Places items using the **First Fit Decreasing Height** shelf algorithm. Tries to fit each item on the first existing shelf with sufficient remaining width before opening a new one.

- `items: list[dict]`
- `bin_width: int | float`
- `start_y: float` — y-offset for the first shelf

Returns `list[tuple[x, y, width, height]]`.

#### `nfdh(rectangles, bin_width)`

Convenience wrapper: calls `expand_items` then `place_items_nfdh`.

- `rectangles: list[tuple[int, int, int]]`

Returns `list[tuple[x, y, width, height]]`.

#### `ffdh(rectangles, bin_width)`

Convenience wrapper: calls `expand_items` then `place_items_ffdh`.

- `rectangles: list[tuple[int, int, int]]`

Returns `list[tuple[x, y, width, height]]`.

#### `find_bottom_left_position(placements, bin_width, width, height, max_height=None)`

Finds the bottom-left feasible position for a rectangle given existing placements (no obstacles).

- `placements: list[tuple[x, y, width, height]]` — already placed items
- `bin_width: int | float`
- `width, height: int | float` — size of the item to place
- `max_height: float | None` — reject positions above this y

Returns `(x, y)` or `None` if no feasible position exists.

---

### Fractional Strip Packing (LP-based)

Used by algorithms that solve the fractional strip cover relaxation before rounding.

#### `get_configurations(type_keys, bin_width)`

Generates all valid one-strip configurations: count vectors `[c0, c1, ...]` where each `ci` items of type `i` fit within `bin_width`.

- `type_keys: list[tuple[int, int]]`
- `bin_width: int`

Returns a sorted list of count vectors. May be expensive with many types or wide bins.

#### `configuration_matrix(configurations, num_types)`

Converts a list of configurations into a 2D matrix with shape `(num_types, num_configurations)`.

#### `solve_fractional_strip_cover(rectangle_list, bin_width)`

Solves the LP relaxation of the strip cover problem using SciPy's HiGHS solver.

- `rectangle_list` — any format accepted by `get_type_keys`
- `bin_width: int`

Returns `(type_keys, configurations, strip_heights)` where `strip_heights[i]` is the fractional height allocated to `configurations[i]`.

Raises `RuntimeError` if the LP fails.

#### `place_strip_band(type_keys, counts, strip_height, remaining_quantities, start_y=0.0, eps=1e-9)`

Places one band of items corresponding to a single LP configuration.

- `type_keys: list[tuple[int, int]]`
- `counts: list[int]` — one count per type for this configuration
- `strip_height: float` — allocated height of the band
- `remaining_quantities: list[int]` — mutated in-place as items are consumed
- `start_y: float`

Returns `(placements, band_height)`.

---

### Obstacle/Blocker API (Repacking)

Used when placing items into a bin that already has occupied regions.

A **blocker** is a dict with keys `x_1`, `x_2`, `y_1`, `y_2` (floats). Blockers are axis-aligned rectangles that items must not overlap.

#### `normalize_obstacles(non_empty_space)`

Converts the `non_empty_space` argument from `Repacking.solve` into a list of blocker dicts with float values.

- Input: list of dicts or objects with `x_1`, `x_2`, `y_1`, `y_2` attributes

Returns `list[dict]` with `"x_1"`, `"x_2"`, `"y_1"`, `"y_2"` keys.

```python
blockers = packing_lib.normalize_obstacles(non_empty_space)
```

#### `placement_to_blocker(placement)`

Converts a placement tuple `(x, y, width, height)` into a blocker dict.

#### `append_placement_as_blocker(blockers, placement)`

Appends a placement as a blocker to an existing blocker list. Call this after each successful placement to prevent future items from overlapping it.

```python
placement = (x, y, width, height)
placements.append(placement)
packing_lib.append_placement_as_blocker(blockers, placement)
```

#### `rectangles_intersect(a, b)`

Returns `True` if two blocker dicts overlap (open boundary check).

#### `can_place_in_region(x, y, width, height, bin_width, bin_height, blockers)`

Returns `True` if a rectangle can be placed at `(x, y)` without leaving the bin or overlapping any blocker.

#### `candidate_x_positions_for_band(blockers, y, band_height)`

Returns sorted candidate x-positions to try when placing in a horizontal band at `[y, y+band_height)`. Includes `0.0` and the left/right edges of any blockers that intersect the band.

#### `leftmost_feasible_x_for_band(blockers, band_y, band_height, width, bin_width, bin_height)`

Finds the smallest feasible x such that a rectangle of the given `width` fits in the band `[band_y, band_y+band_height)` without overlapping any blocker or the bin boundary.

Returns `int` or `None` if no position exists.

#### `next_level_y(blockers, start_y, level_height, bin_width, bin_height)`

Finds the lowest y-position ≥ `start_y` where a shelf of the given `level_height` can be opened (i.e., at least one x-position is feasible).

Returns `int` or `None` if no valid level exists.

#### `find_bottom_left_position_with_obstacles(blockers, bin_width, bin_height, width, height)`

Finds the bottom-left feasible position for a rectangle in a fixed bin with obstacles, analogous to `find_bottom_left_position` but respecting the bin height and blockers.

Returns `(x, y)` or `None`.

---

## Examples

### Minimal strip packer

```python
import packing_lib

class Packing:
    def solve(self, bin_width, rectangles):
        items = packing_lib.sort_by_height(packing_lib.expand_items(rectangles))
        placements = packing_lib.place_items_ffdh(items, bin_width)
        return packing_lib.output_from_placements(bin_width, placements)
```

### Minimal repacker with obstacles

```python
import packing_lib

class Repacking:
    def solve(self, bin_height, bin_width, rectangles, non_empty_space):
        items = packing_lib.sort_by_height(packing_lib.expand_items(rectangles))
        blockers = packing_lib.normalize_obstacles(non_empty_space)
        placements = []

        for item in items:
            x = packing_lib.leftmost_feasible_x_for_band(
                blockers, 0, item["height"], item["width"], bin_width, bin_height
            )
            if x is None:
                raise ValueError("Could not place item")
            placement = (x, 0, item["width"], item["height"])
            placements.append(placement)
            packing_lib.append_placement_as_blocker(blockers, placement)

        return packing_lib.output_from_placements(bin_width, placements)
```

---

## Notes

- `make_output` and `output_from_placements` return a JSON **string**, not a dict.
- `sort_by_quantity` expects dict inputs with a `"quantity"` key — not `[w, h, q]` tuples.
- `expand_items` drops quantity information; use `expand_type_counts` if you need to reconstruct items from `type_keys` and a (possibly modified) quantities array.
- `get_configurations` is combinatorial and can be slow with many rectangle types or large bin widths.
- `solve_fractional_strip_cover` requires `scipy` to be available in the runtime environment.
- `append_placement_as_blocker` mutates the blocker list in-place; call it immediately after each successful placement.
