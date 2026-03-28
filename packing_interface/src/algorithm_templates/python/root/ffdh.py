import packing_lib
from typing import List, Tuple

class Packing:
     def solve(self, bin_width: int, rectangles: List[Tuple[int, int, int]]) -> List[Tuple[float, float, int, int]]:       
        items = packing_lib.expand_items(rectangles)
        items = packing_lib.sort_by_height(items)
        levels = []
        placements = []
        current_y = 0

        for item in items:
            width = item["width"]
            height = item["height"]
            placed = False

            for level in levels:
                if level["used_width"] + width <= bin_width:
                    placements.append((level["used_width"], level["y"], width, height))
                    level["used_width"] += width
                    placed = True
                    break

            if placed:
                continue

            levels.append({"height": height, "used_width": width, "y": current_y})
            placements.append((0, current_y, width, height))
            current_y += height
        return packing_lib.output_from_placements(bin_width, placements)
