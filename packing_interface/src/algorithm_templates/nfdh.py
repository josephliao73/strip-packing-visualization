import packing_lib
import json
from typing import List, Tuple

class Packing:
    def solve(self, bin_width: int, rectangles: List[Tuple[int, int, int]]) -> List[Tuple[float, float, int, int]]:
        items = packing_lib.expand_items(rectangles)
        items = packing_lib.sort_by_height(items)
        placements = []
        current_level_y = 0 
        current_level_height = 0
        current_level_used_width = 0

        for item in items:
            width = item["width"]
            height = item["height"]

            if current_level_used_width == 0:
                current_level_height = height

            if current_level_used_width + width <= bin_width:
                placements.append((current_level_used_width, current_level_y, width, height))
                current_level_used_width += width
                continue

            current_level_y += current_level_height
            current_level_height = height
            current_level_used_width = width
            placements.append((0, current_level_y, width, height))
        return packing_lib.output_from_placements(bin_width, placements)
