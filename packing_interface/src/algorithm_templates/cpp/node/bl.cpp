#include "packing_lib.h"
using namespace packing;

class Repacking {
public:
    std::vector<std::tuple<double, double, int, int>> solve(
        int binHeight,
        int binWidth,
        const std::vector<std::tuple<int, int, int>>& rectangles,
        const std::vector<Obstacle>& obstacles
    ) {
        auto items = sort_by_width(expand_items(rectangles));
        std::vector<Obstacle> blockers = obstacles;
        std::vector<std::tuple<double, double, int, int>> placements;

        for (const auto& item : items) {
            auto position = find_bottom_left_position_with_obstacles(blockers, binWidth, binHeight, item.width, item.height);
            if (!position.has_value()) {
                throw std::runtime_error("Unable to place rectangle with BL node packing");
            }

            auto placement = std::make_tuple(position->first, position->second, item.width, item.height);
            placements.push_back(placement);
            append_placement_as_obstacle(blockers, placement);
        }

        return placements;
    }
};
