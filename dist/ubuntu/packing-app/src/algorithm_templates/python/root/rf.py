import packing_lib
from typing import List, Tuple


class Packing:
    def solve(self, bin_width: int, rectangles: List[Tuple[int, int, int]]) -> List[Tuple[float, float, int, int]]:
        items = packing_lib.expand_items(rectangles)

        wide_items = [item for item in items if item["width"] > bin_width / 2]
        narrow_items = packing_lib.sort_by_height([item for item in items if item["width"] <= bin_width / 2])

        placements = []
        h0 = 0.0

        for item in wide_items:
            placements.append((0.0, h0, item["width"], item["height"]))
            h0 += item["height"]

        idx = 0
        while idx < len(narrow_items):
            # Forward level: pack L→R at h0
            forward = []
            x = 0.0
            while idx < len(narrow_items):
                item = narrow_items[idx]
                if x + item["width"] > bin_width:
                    break
                forward.append((x, h0, item["width"], item["height"]))
                x += item["width"]
                idx += 1

            if not forward:
                break

            for p in forward:
                placements.append(p)

            # forward[0] is the tallest item (sorted by non-increasing height)
            h_f1 = forward[0][3]
            forward_top = h0 + h_f1

            # Reverse level: pack R→L until cumulative width >= bin_width / 2
            reverse = []
            x_right = float(bin_width)
            cumulative_width = 0.0
            while idx < len(narrow_items) and cumulative_width < bin_width / 2:
                item = narrow_items[idx]
                x_right -= item["width"]
                reverse.append((x_right, item["width"], item["height"]))
                cumulative_width += item["width"]
                idx += 1

            if reverse:
                # Drop the reverse level as a unit. Items start with bottoms at forward_top
                # and are lowered by max_delta without overlapping any forward-level item.
                # Constraint per overlapping (reverse, forward) x-pair: delta <= h_f1 - fh.
                max_delta = float(h_f1)
                for (rx, rw, _rh) in reverse:
                    for (fx, _fy, fw, fh) in forward:
                        if rx < fx + fw and rx + rw > fx:
                            max_delta = min(max_delta, h_f1 - fh)

                y_base = forward_top - max_delta
                for (rx, rw, rh) in reverse:
                    placements.append((rx, y_base, rw, rh))

                reverse_top = max(y_base + rh for (_rx, _rw, rh) in reverse)
                h0 = max(forward_top, reverse_top)
            else:
                h0 = forward_top

        return packing_lib.output_from_placements(bin_width, placements)
