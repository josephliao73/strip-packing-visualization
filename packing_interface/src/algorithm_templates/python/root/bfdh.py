import packing_lib
from typing import List, Tuple


class Packing:
    def solve(self, bin_width: int, rectangles: List[Tuple[int, int, int]]) -> List[Tuple[float, float, int, int]]:
        items = packing_lib.sort_by_height(packing_lib.expand_items(rectangles))
        levels = []
        placements = []
        current_y = 0

        for item in items:
            width = item["width"]
            height = item["height"]

            if width > bin_width:
                raise ValueError(f"Rectangle width {width} exceeds bin width {bin_width}.")

            best_level = None
            best_remaining_width = None

            for level in levels:
                if level["used_width"] + width > bin_width:
                    continue

                remaining_width = bin_width - (level["used_width"] + width)
                if best_remaining_width is None or remaining_width < best_remaining_width:
                    best_level = level
                    best_remaining_width = remaining_width

            if best_level is not None:
                placements.append((best_level["used_width"], best_level["y"], width, height))
                best_level["used_width"] += width
                continue

            levels.append({"y": current_y, "height": height, "used_width": width})
            placements.append((0, current_y, width, height))
            current_y += height

        return packing_lib.output_from_placements(bin_width, placements)
