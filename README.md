# Bin Packing Visualization Interface

Desktop application for creating 2D bin packing problem instances and visualizing their solutions.

## Features

### Input Configuration
- **Manual Rectangle Entry**: Define rectangles with width, height, and quantity
- **File Import**: Load rectangle configurations from `.txt`, `.in`, or `.csv` files
- **Autofill Mode**: Automatically generate additional rectangles to meet specified constraints
- **Input Validation**: Real-time validation with detailed error messages
- **Export to JSON**: Generate algorithm-ready input files

### Visualization
- **Algorithm Output Import**: Load JSON files containing placement solutions
- **Animated Display**: Watch rectangles appear one-by-one with adjustable animation speed
- **Interactive Canvas**:
  - **Zoom**: Mouse wheel to zoom in/out (10% - 1000%)
  - **Pan**: Click and drag outside the bin to move the view
  - **Rectangle Dragging**: Click and drag rectangles to reposition them
  - **Hover Information**: See rectangle dimensions when hovering
- **Smart Snapping**: Rectangles snap to bin edges when dragged close
- **Collision Detection**: Visual feedback (green/red borders) for valid/invalid placements
- **Height Recalculation**: Bin height automatically adjusts when rectangles are moved

## Installation

### Prerequisites
- **Rust**: Install from [rust-lang.org](https://www.rust-lang.org/tools/install)


```bash
cd packing_interface

cargo build --release

cargo run --release
```


### Creating Algorithm Input

The application has the option that helps you create standardized input files for bin packing algorithms:

1. **Set Bin Width** (Required): The fixed width of the bin
2. **Set Rectangle Count** (Optional): Total number of rectangles (N)
3. **Set Rectangle Types** (Optional): Number of unique rectangle dimensions (K)
4. **Enable Autofill** (Optional): Let the app generate random rectangles to meet N and K constraints

#### Rectangle Data Format

Enter rectangles in the text editor using this format:
```
X Y Q
```
- **X**: Width of the rectangle
- **Y**: Height of the rectangle
- **Q**: Quantity of the type of rectangle

**Example:**
```
10 20 5
15 15 3
8 25 2
```
This defines:
- 5 rectangles of size 10 by 20
- 3 rectangles of size 15 by 15
- 2 rectangles of size 8 by 25

#### Autofill 

When autofill is enabled:
- If K types is specified and current types < K then generates new random rectangle types within the bin width and heights are randomly chosen within the range of existing rectangle heights
- If N quantity is specified and current quantity < N then it randomly increases quantities of existing rectangles and distributes additional rectangles across the types

#### Export

"Export Algorithm Input" button to save a JSON file:
```json
{
  "width_of_bin": 100,
  "number_of_rectangles": 150,
  "number_of_types_of_rectangles": 12,
  "autofill_option": true,
  "rectangle_list": [
    {"width": 10, "height": 20, "quantity": 5},
    {"width": 15, "height": 15, "quantity": 3}
  ]
}
```

### Visualizing Algorithm Output

After your bin packing algorithm processes the input, visualize the results:

1. Click **"Import Output JSON"**
2. Select a JSON file with this structure:
```json
{
  "bin_width": 100,
  "total_height": 250.5,
  "placements": [
    {"x": 0.0, "y": 0.0, "width": 10, "height": 20},
    {"x": 10.0, "y": 0.0, "width": 15, "height": 15}
  ]
}
```

#### Placement Format
- **x**: X-coordinate (left edge of rectangle)
- **y**: Y-coordinate (bottom edge of rectangle)
- **width**: Rectangle width
- **height**: Rectangle height


## File Format Examples

### Input Configuration File (`.txt`, `.in`, `.csv`)
```
# Comments are ignored
10 20 5
15 15 3
8 25 2
```

### Algorithm Input JSON (exported)
```json
{
  "width_of_bin": 100,
  "number_of_rectangles": 10,
  "number_of_types_of_rectangles": 3,
  "autofill_option": false,
  "rectangle_list": [
    {"width": 10, "height": 20, "quantity": 5},
    {"width": 15, "height": 15, "quantity": 3},
    {"width": 8, "height": 25, "quantity": 2}
  ]
}
```

### Algorithm Output JSON (for visualization)
```json
{
  "bin_width": 100,
  "total_height": 65.0,
  "placements": [
    {"x": 0.0, "y": 0.0, "width": 10, "height": 20},
    {"x": 10.0, "y": 0.0, "width": 15, "height": 15},
    {"x": 25.0, "y": 0.0, "width": 8, "height": 25},
    {"x": 0.0, "y": 20.0, "width": 10, "height": 20},
    {"x": 10.0, "y": 15.0, "width": 15, "height": 15}
  ]
}
```
