#include "packing_lib.h"
using namespace packing;

class Repacking {
public:
    // rectangles: each element is (width, height, quantity)
    // obstacles:  Obstacle { x1, x2, y1, y2 } regions already occupied
    // returns:    placements as (x, y, width, height)
    std::vector<std::tuple<double, double, int, int>> solve(
        int binHeight,
        int binWidth,
        const std::vector<std::tuple<int, int, int>>& rectangles,
        const std::vector<Obstacle>& obstacles
    ) {
        auto items = expand_items(rectangles);

        std::vector<std::tuple<double, double, int, int>> placements;
        double y = 0.0;
        for (const auto& item : items) {
            placements.push_back({0.0, y, item.width, item.height});
            y += item.height;
        }
        return placements;
    }
};
