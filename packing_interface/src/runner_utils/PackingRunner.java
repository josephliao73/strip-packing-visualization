
import java.util.*;
import java.util.regex.*;

public class PackingRunner {

    public static void main(String[] args) {
        if (args.length != 2) {
            System.err.println("Usage: java PackingRunner <bin_width> <rectangles_json>");
            System.exit(1);
        }

        int binWidth = Integer.parseInt(args[0]);
        String rectanglesJson = args[1];

        List<int[]> rectangles = parseRectangles(rectanglesJson);

        Packing packing = new Packing();
        List<double[]> placements = packing.solve(binWidth, rectangles);

        double totalHeight = 0.0;
        for (double[] p : placements) {
            double top = p[1] + p[3];
            if (top > totalHeight) totalHeight = top;
        }

        StringBuilder sb = new StringBuilder();
        sb.append("{\"bin_width\":").append(binWidth);
        sb.append(",\"total_height\":").append(totalHeight);
        sb.append(",\"placements\":[");

        for (int i = 0; i < placements.size(); i++) {
            if (i > 0) sb.append(",");
            double[] p = placements.get(i);
            sb.append("{\"x\":").append(p[0]);
            sb.append(",\"y\":").append(p[1]);
            sb.append(",\"width\":").append((int)p[2]);
            sb.append(",\"height\":").append((int)p[3]).append("}");
        }

        sb.append("]}");
        System.out.println(sb.toString());
    }

    private static List<int[]> parseRectangles(String json) {
        List<int[]> result = new ArrayList<>();
        Pattern pattern = Pattern.compile("\\[(\\d+)\\s*,\\s*(\\d+)\\s*,\\s*(\\d+)\\]");
        Matcher matcher = pattern.matcher(json);

        while (matcher.find()) {
            int w = Integer.parseInt(matcher.group(1));
            int h = Integer.parseInt(matcher.group(2));
            int q = Integer.parseInt(matcher.group(3));
            result.add(new int[]{w, h, q});
        }

        return result;
    }
}
