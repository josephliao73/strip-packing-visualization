#include "packing_lib.h"
using namespace packing;

class Packing {
public:
    std::vector<std::tuple<double, double, int, int>> solve(
        int binWidth,
        const std::vector<std::tuple<int, int, int>>& rectangles
    ) {
        auto items = sort_by_width(expand_items(rectangles));
        std::vector<std::tuple<double, double, int, int>> placements;

        for (const auto& item : items) {
            if (item.width > binWidth) {
                throw std::runtime_error("Rectangle width exceeds bin width.");
            }

            auto position = find_bottom_left_position(placements, binWidth, item.width, item.height);
            if (!position.has_value()) {
                throw std::runtime_error("Unable to place rectangle with Bottom-Left heuristic.");
            }

            placements.push_back({position->first, position->second, item.width, item.height});
        }

        return placements;
    }
};
