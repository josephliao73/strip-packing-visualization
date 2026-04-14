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
        std::vector<Rect> narrowItems;

        for (const auto& item : items) {
            if (item.width > binWidth / 2.0) {
                wideItems.push_back(item);
            } else {
                narrowItems.push_back(item);
            }
        }
        narrowItems = sort_by_height(std::move(narrowItems));

        std::vector<std::tuple<double, double, int, int>> placements;
        double h0 = 0.0;

        for (const auto& item : wideItems) {
            placements.push_back({0.0, h0, item.width, item.height});
            h0 += item.height;
        }

        struct ForwardItem { double x, y; int width, height; };
        struct ReverseItem { double x; int width, height; };

        std::size_t idx = 0;
        while (idx < narrowItems.size()) {
            // Forward level: pack L→R at h0
            std::vector<ForwardItem> forward;
            double x = 0.0;
            while (idx < narrowItems.size()) {
                const auto& item = narrowItems[idx];
                if (x + item.width > binWidth) break;
                forward.push_back({x, h0, item.width, item.height});
                x += item.width;
                ++idx;
            }

            if (forward.empty()) break;

            for (const auto& f : forward) {
                placements.push_back({f.x, f.y, f.width, f.height});
            }

            // forward[0] is the tallest item (sorted by non-increasing height)
            double hf1 = static_cast<double>(forward[0].height);
            double forwardTop = h0 + hf1;

            // Reverse level: pack R→L until cumulative width >= binWidth / 2
            std::vector<ReverseItem> reverse;
            double xRight = static_cast<double>(binWidth);
            double cumulativeWidth = 0.0;
            while (idx < narrowItems.size() && cumulativeWidth < binWidth / 2.0) {
                const auto& item = narrowItems[idx];
                xRight -= item.width;
                reverse.push_back({xRight, item.width, item.height});
                cumulativeWidth += item.width;
                ++idx;
            }

            if (!reverse.empty()) {
                // Drop the reverse level as a unit. Items start with bottoms at forwardTop
                // and are lowered by maxDelta without overlapping any forward-level item.
                // Constraint per overlapping (reverse, forward) x-pair: delta <= hf1 - fh.
                double maxDelta = hf1;
                for (const auto& r : reverse) {
                    for (const auto& f : forward) {
                        if (r.x < f.x + f.width && r.x + r.width > f.x) {
                            maxDelta = std::min(maxDelta, hf1 - static_cast<double>(f.height));
                        }
                    }
                }

                double yBase = forwardTop - maxDelta;
                for (const auto& r : reverse) {
                    placements.push_back({r.x, yBase, r.width, r.height});
                }

                double reverseTop = 0.0;
                for (const auto& r : reverse) {
                    reverseTop = std::max(reverseTop, yBase + r.height);
                }
                h0 = std::max(forwardTop, reverseTop);
            } else {
                h0 = forwardTop;
            }
        }

        return placements;
    }
};
