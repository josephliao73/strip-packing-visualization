    import packing_lib
import json
from typing import List, Tuple

class Packing:
    def solve(self, bin_width: int, rectangles: List[Tuple[int, int, int]]) -> List[Tuple[float, float, int, int]]:
        items = packing_lib.expand_items(rectangles)

        placements = [] 

        return packing_lib.output_from_placements(bin_width, placements)
