#include "packing_lib.h"
using namespace packing;

class Packing {
public:
    std::vector<std::tuple<double, double, int, int>> solve(
        int binWidth,
        const std::vector<std::tuple<int, int, int>>& rectangles
    ) {
        auto items = expand_items(rectangles);
        std::vector<Rect> wideItems;
        std::vector<Rect> remainingItems;

        for (const auto& item : items) {
            if (item.width > binWidth) {
                throw std::runtime_error("Rectangle width exceeds bin width.");
            }
            if (item.width > binWidth / 2.0) {
                wideItems.push_back(item);
            } else {
                remainingItems.push_back(item);
            }
        }

        std::vector<std::tuple<double, double, int, int>> placements;
        double currentY = 0.0;

        for (const auto& item : wideItems) {
            placements.push_back({0.0, currentY, item.width, item.height});
            currentY += item.height;
        }

        remainingItems = sort_by_height(std::move(remainingItems));
        double halfWidth = static_cast<double>(binWidth) / 2.0;
        double leftBaseline = currentY;
        double rightBaseline = currentY;

        std::vector<std::pair<double, double>> levelBuffer;
        double levelUsedWidth = 0.0;
        double levelHeight = 0.0;

        auto appendFfdhFallback = [&](const std::vector<Rect>& fallbackItems, double startY) {
            struct Level {
                double y;
                int height;
                int usedWidth;
            };

            std::vector<Level> levels;
            double currentFallbackY = startY;
            for (const auto& fallbackItem : fallbackItems) {
                bool placed = false;
                for (auto& level : levels) {
                    if (level.usedWidth + fallbackItem.width > binWidth) {
                        continue;
                    }
                    placements.push_back({
                        static_cast<double>(level.usedWidth),
                        level.y,
                        fallbackItem.width,
                        fallbackItem.height,
                    });
                    level.usedWidth += fallbackItem.width;
                    placed = true;
                    break;
                }

                if (placed) {
                    continue;
                }

                levels.push_back({currentFallbackY, fallbackItem.height, fallbackItem.width});
                placements.push_back({0.0, currentFallbackY, fallbackItem.width, fallbackItem.height});
                currentFallbackY += fallbackItem.height;
            }
        };

        auto flushFullWidthLevel = [&]() {
            if (levelBuffer.empty()) {
                return;
            }
            double x = 0.0;
            for (const auto& [width, height] : levelBuffer) {
                placements.push_back({x, currentY, static_cast<int>(width), static_cast<int>(height)});
                x += width;
            }
            currentY += levelHeight;
            leftBaseline = currentY;
            rightBaseline = currentY;
            levelBuffer.clear();
            levelUsedWidth = 0.0;
            levelHeight = 0.0;
        };

        std::size_t idx = 0;
        while (idx < remainingItems.size()) {
            const auto& item = remainingItems[idx];
            if (levelUsedWidth + item.width > binWidth) {
                break;
            }
            levelBuffer.push_back({static_cast<double>(item.width), static_cast<double>(item.height)});
            levelUsedWidth += item.width;
            levelHeight = std::max(levelHeight, static_cast<double>(item.height));
            ++idx;
        }

        flushFullWidthLevel();

        while (idx < remainingItems.size()) {
            bool placeLeft = leftBaseline <= rightBaseline;
            double baseline = placeLeft ? leftBaseline : rightBaseline;
            double xStart = placeLeft ? 0.0 : halfWidth;
            double usedWidth = 0.0;
            double halfLevelHeight = 0.0;
            std::vector<std::pair<double, double>> levelItems;

            while (idx < remainingItems.size()) {
                const auto& item = remainingItems[idx];
                if (item.width > halfWidth || usedWidth + item.width > halfWidth) {
                    break;
                }
                levelItems.push_back({static_cast<double>(item.width), static_cast<double>(item.height)});
                usedWidth += item.width;
                halfLevelHeight = std::max(halfLevelHeight, static_cast<double>(item.height));
                ++idx;
            }

            if (levelItems.empty()) {
                auto fallbackItems = std::vector<Rect>(remainingItems.begin() + static_cast<long>(idx), remainingItems.end());
                double fallbackY = std::max(leftBaseline, rightBaseline);
                appendFfdhFallback(fallbackItems, fallbackY);
                break;
            }

            double x = xStart;
            for (const auto& [width, height] : levelItems) {
                placements.push_back({x, baseline, static_cast<int>(width), static_cast<int>(height)});
                x += width;
            }

            if (placeLeft) {
                leftBaseline = baseline + halfLevelHeight;
            } else {
                rightBaseline = baseline + halfLevelHeight;
            }
        }

        return placements;
    }
};
