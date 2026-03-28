import packing_lib
from typing import List, Tuple


class Packing:
    def solve(self, bin_width: int, rectangles: List[Tuple[int, int, int]]) -> List[Tuple[float, float, int, int]]:
        items = packing_lib.expand_items(rectangles)
        items = packing_lib.sort_by_width(items)
        placements = []

        for item in items:
            width = item["width"]
            height = item["height"]

            if width > bin_width:
                raise ValueError(f"Rectangle width {width} exceeds bin width {bin_width}.")

            position = packing_lib.find_bottom_left_position(
                placements,
                bin_width,
                width,
                height,
            )
            if position is None:
                raise ValueError("Unable to place rectangle with Bottom-Left heuristic.")

            x, y = position
            placements.append((x, y, width, height))

        return packing_lib.output_from_placements(bin_width, placements)
