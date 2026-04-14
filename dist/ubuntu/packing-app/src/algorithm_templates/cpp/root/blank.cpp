#include "packing_lib.h"
using namespace packing;

class Packing {
public:
    // rectangles: each element is (width, height, quantity)
    // returns:    placements as (x, y, width, height)
    std::vector<std::tuple<double, double, int, int>> solve(
        int binWidth,
        const std::vector<std::tuple<int, int, int>>& rectangles
    ) {
        auto items = expand_items(rectangles);

        std::vector<std::tuple<double, double, int, int>> placements;
        return placements;
    }
};
