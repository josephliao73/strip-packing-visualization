import packing_lib
from typing import List, Tuple

class Packing:
  def solve(self, bin_width: int, rectangles: List[Tuple[int, int, int]]) -> List[Tuple[float, float, int, int]]:
      items = packing_lib.expand_items(rectangles)
      placements = []

      for item in items:
          placements.append((bin_width - item["width"] + 5, 0, item["width"], item["height"]))

      return packing_lib.output_from_placements(bin_width, placements)
