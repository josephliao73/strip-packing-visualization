#include "packing_lib.h"
using namespace packing;

class Packing {
public:
    std::vector<std::tuple<double, double, int, int>> solve(
        int binWidth,
        const std::vector<std::tuple<int, int, int>>& rectangles
    ) {
        auto items = sort_by_width(expand_items(rectangles));

        struct Region {
            std::vector<std::tuple<double, double, int, int>> placements;
            double height = 0.0;
        };

        std::vector<Region> regions(5);
        std::vector<Rect> largeGroups[4];
        std::vector<Rect> smallItems;

        for (const auto& item : items) {
            if (item.width > binWidth) {
                throw std::runtime_error("Rectangle width exceeds bin width.");
            }

            double ratio = binWidth > 0 ? static_cast<double>(item.width) / binWidth : 1.0;
            if (ratio > 0.5) {
                largeGroups[0].push_back(item);
            } else if (ratio > (1.0 / 3.0)) {
                largeGroups[1].push_back(item);
            } else if (ratio > 0.25) {
                largeGroups[2].push_back(item);
            } else if (ratio > 0.2) {
                largeGroups[3].push_back(item);
            } else {
                smallItems.push_back(item);
            }
        }

        for (int groupIndex = 0; groupIndex < 4; ++groupIndex) {
            auto& ownRegion = regions[groupIndex];
            for (const auto& item : largeGroups[groupIndex]) {
                bool placed = false;

                for (int priorIndex = 0; priorIndex < groupIndex; ++priorIndex) {
                    auto& priorRegion = regions[priorIndex];
                    auto position = find_bottom_left_position(
                        priorRegion.placements,
                        binWidth,
                        item.width,
                        item.height,
                        priorRegion.height
                    );
                    if (!position.has_value()) {
                        continue;
                    }

                    priorRegion.placements.push_back({position->first, position->second, item.width, item.height});
                    placed = true;
                    break;
                }

                if (placed) {
                    continue;
                }

                auto position = find_bottom_left_position(ownRegion.placements, binWidth, item.width, item.height);
                if (!position.has_value()) {
                    throw std::runtime_error("Unable to place rectangle with the Up-Down heuristic.");
                }

                ownRegion.placements.push_back({position->first, position->second, item.width, item.height});
                ownRegion.height = std::max(ownRegion.height, position->second + item.height);
            }
        }

        smallItems = sort_by_height(std::move(smallItems));
        struct Shelf {
            double y;
            int height;
            int usedWidth;
        };
        std::vector<Shelf> regionFiveShelves;

        for (const auto& item : smallItems) {
            bool placed = false;

            for (int priorIndex = 0; priorIndex < 4; ++priorIndex) {
                auto& priorRegion = regions[priorIndex];
                auto position = find_bottom_left_position(
                    priorRegion.placements,
                    binWidth,
                    item.width,
                    item.height,
                    priorRegion.height
                );
                if (!position.has_value()) {
                    continue;
                }

                priorRegion.placements.push_back({position->first, position->second, item.width, item.height});
                placed = true;
                break;
            }

            if (placed) {
                continue;
            }

            for (auto& shelf : regionFiveShelves) {
                if (shelf.usedWidth + item.width > binWidth || item.height > shelf.height) {
                    continue;
                }

                double x = static_cast<double>(shelf.usedWidth);
                shelf.usedWidth += item.width;
                regions[4].placements.push_back({x, shelf.y, item.width, item.height});
                placed = true;
                break;
            }

            if (placed) {
                continue;
            }

            double shelfY = 0.0;
            for (const auto& shelf : regionFiveShelves) {
                shelfY += shelf.height;
            }
            regionFiveShelves.push_back({shelfY, item.height, item.width});
            regions[4].placements.push_back({0.0, shelfY, item.width, item.height});
            regions[4].height = std::max(regions[4].height, shelfY + item.height);
        }

        std::vector<std::tuple<double, double, int, int>> placements;
        double yOffset = 0.0;
        for (const auto& region : regions) {
            for (const auto& [x, y, width, height] : region.placements) {
                placements.push_back({x, y + yOffset, width, height});
            }
            yOffset += region.height;
        }

        return placements;
    }
};
