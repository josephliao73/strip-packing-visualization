import packing_lib
import json
from typing import List, Tuple

class Packing:
    def solve(self, bin_width: int, rectangles: List[Tuple[int, int, int]]) -> List[Tuple[float, float, int, int]]:

        type_keys, remaining = packing_lib.get_type_keys(rectangles)
        _, configurations, strip_heights = packing_lib.solve_fractional_strip_cover(rectangles, bin_width)

        placements = []
        current_y = 0.0

        for config, strip_height in zip(configurations, strip_heights):
            if strip_height <= 1e-9:
                continue

            band_placements, band_height = packing_lib.place_strip_band(
                type_keys,
                config,
                strip_height,
                remaining,
                current_y,
            )
            placements.extend(band_placements)
            current_y += band_height

        leftovers = packing_lib.expand_type_counts(type_keys, remaining)
        placements.extend(packing_lib.place_items_ffdh(leftovers, bin_width, current_y))

        return packing_lib.output_from_placements(bin_width, placements)
