#include "packing_lib.h"
using namespace packing;

class Packing {
public:
    std::vector<std::tuple<double, double, int, int>> solve(
        int binWidth,
        const std::vector<std::tuple<int, int, int>>& rectangles
    ) {
        auto items = sort_by_height(expand_items(rectangles));

        std::vector<std::tuple<double, double, int, int>> placements;
        int currentLevelY = 0;
        int currentLevelHeight = 0;
        int currentLevelUsedWidth = 0;

        for (const auto& item : items) {
            if (currentLevelUsedWidth == 0) {
                currentLevelHeight = item.height;
            }

            if (currentLevelUsedWidth + item.width <= binWidth) {
                placements.push_back({
                    static_cast<double>(currentLevelUsedWidth),
                    static_cast<double>(currentLevelY),
                    item.width,
                    item.height,
                });
                currentLevelUsedWidth += item.width;
                continue;
            }

            currentLevelY += currentLevelHeight;
            currentLevelHeight = item.height;
            currentLevelUsedWidth = item.width;
            placements.push_back({0.0, static_cast<double>(currentLevelY), item.width, item.height});
        }

        return placements;
    }
};
