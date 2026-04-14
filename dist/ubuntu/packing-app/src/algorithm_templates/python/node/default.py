import packing_lib


class Repacking:
    def solve(self, bin_height, bin_width, rectangles, non_empty_space):
        placements = []
        return packing_lib.output_from_placements(bin_width, placements)
