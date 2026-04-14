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
        auto items = sort_by_height(expand_items(rectangles));
        std::vector<Obstacle> blockers = obstacles;
        std::vector<std::tuple<double, double, int, int>> placements;
        std::optional<int> currentY;
        int currentHeight = 0;

        for (const auto& item : items) {
            if (!currentY.has_value()) {
                currentHeight = item.height;
                currentY = next_level_y(blockers, 0, currentHeight, binWidth, binHeight);
                if (!currentY.has_value()) {
                    throw std::runtime_error("Unable to open initial NFDH node level");
                }
            }

            auto x = leftmost_feasible_x_for_band(blockers, *currentY, currentHeight, item.width, binWidth, binHeight);
            if (!x.has_value()) {
                int previousHeight = currentHeight;
                currentHeight = item.height;
                currentY = next_level_y(blockers, *currentY + previousHeight, item.height, binWidth, binHeight);
                if (currentY.has_value()) {
                    x = leftmost_feasible_x_for_band(blockers, *currentY, currentHeight, item.width, binWidth, binHeight);
                }
            }

            std::tuple<double, double, int, int> placement;
            if (!x.has_value()) {
                auto position = find_bottom_left_position_with_obstacles(blockers, binWidth, binHeight, item.width, item.height);
                if (!position.has_value()) {
                    throw std::runtime_error("Unable to place rectangle with NFDH node packing");
                }
                placement = std::make_tuple(position->first, position->second, item.width, item.height);
            } else {
                placement = std::make_tuple(*x, static_cast<double>(*currentY), item.width, item.height);
            }
            placements.push_back(placement);
            append_placement_as_obstacle(blockers, placement);
        }

        return placements;
    }
};
