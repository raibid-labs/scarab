# Graphics Inspector

The Graphics Inspector is a debugging tool for monitoring inline terminal images rendered using Sixel, Kitty, and iTerm2 protocols.

## Features

- **Live Image Tracking**: View all active image placements in real-time
- **Protocol Detection**: Automatically identifies image format (PNG, JPEG, GIF, RGBA/Sixel)
- **Position Tracking**: Grid coordinates and screen position mapping
- **Memory Monitoring**: Track memory usage per image and total cache size
- **Metadata Export**: Export image metadata to JSON for debugging

## Opening the Inspector

Press `Ctrl+Shift+G` to toggle the graphics inspector overlay.

## Interface Layout

The inspector window is divided into four sections:

### 1. Toolbar
- **Filter**: Search images by ID or protocol type
- **Sort**: Order by ID, position, size, or protocol
- **Clear Selection**: Reset selected image
- **Close**: Close the inspector window

### 2. Statistics Panel
Displays real-time statistics:
- Total number of images loaded
- Number of currently visible images
- Current memory usage
- Peak memory usage

### 3. Image List (Left Panel)
Shows all active image placements with:
- Image ID
- Protocol type (PNG/JPEG/GIF/RGBA)
- Grid position (x, y)
- Cell dimensions when selected

### 4. Details Panel (Right Panel)
When an image is selected, shows:

#### Basic Information
- Image ID
- Protocol type

#### Position
- Grid position (column, row)
- Cell dimensions (width x height)
- Screen position in pixels

#### Memory
- Shared memory offset
- Compressed data size
- Estimated decoded size (RGBA buffer)

#### Actions
- **Copy Image ID**: Copy to clipboard
- **Copy Position**: Copy grid coordinates
- **Export Metadata**: Save metadata to JSON file

#### Technical Details (Collapsible)
- Raw format enum value
- Exact memory offsets
- Cell dimensions

## Sort Modes

### By ID
Images sorted by unique identifier (chronological order)

### By Position
Images sorted by grid position (top-to-bottom, left-to-right)

### By Size
Images sorted by cell dimensions (largest first)

### By Protocol
Images grouped by format type

## Memory Statistics

The inspector tracks:
- **Total Memory**: Sum of all compressed image data in shared memory
- **Peak Memory**: Highest memory usage seen during session
- **Decoded Size**: Estimated RGBA buffer size after decompression

## Metadata Export

Clicking "Export Metadata" creates a JSON file with:
```json
{
  "id": 42,
  "format": "Png",
  "position": {
    "x": 10,
    "y": 5
  },
  "dimensions": {
    "width_cells": 20,
    "height_cells": 10
  },
  "shared_memory": {
    "offset": 4096,
    "size": 12345
  }
}
```

## Protocol Support

### PNG
Standard PNG images transmitted via iTerm2 or Kitty protocols.

### JPEG
JPEG images with lossy compression.

### GIF
Animated or static GIF images (note: only first frame may be displayed).

### RGBA (Sixel)
Raw RGBA pixel data, typically from Sixel protocol.
The inspector estimates dimensions based on cell count.

## Use Cases

### Debugging Image Rendering
- Verify images are being loaded correctly
- Check if images are positioned as expected
- Monitor memory usage for performance tuning

### Protocol Testing
- Confirm protocol detection is working
- Validate image format conversion
- Test different image transmission methods

### Performance Analysis
- Track memory consumption over time
- Identify memory leaks or cache issues
- Monitor peak usage during heavy image loads

## Keyboard Shortcuts

- `Ctrl+Shift+G`: Toggle inspector
- Mouse scroll: Navigate image list
- Click: Select image for details

## Implementation Details

### Architecture
The graphics inspector integrates with:
- `SharedImageReader`: Reads from shared memory buffer
- `ImageCache`: Accesses decoded texture cache
- `TerminalMetrics`: Converts grid to screen coordinates

### Integration Points
- Located at `/home/beengud/raibid-labs/scarab/crates/scarab-client/src/graphics_inspector.rs`
- Plugin: `GraphicsInspectorPlugin`
- Resource: `GraphicsInspectorState`

### Dependencies
- `bevy_egui`: UI framework
- `serde_json`: Metadata export
- `arboard`: Clipboard operations

## Troubleshooting

### Inspector Won't Open
- Ensure daemon is running and images are enabled
- Check that shared memory buffer is connected
- Verify `Ctrl+Shift+G` keybinding is not conflicting

### No Images Shown
- Confirm images are actually being rendered in terminal
- Check that image protocols are enabled in daemon
- Verify shared memory sequence number is updating

### Statistics Not Updating
- Check if `SharedImageReader` resource exists
- Verify image cache is being populated
- Look for errors in daemon logs

## Future Enhancements

Potential additions:
- Image thumbnail preview
- Protocol-specific details (Sixel color palette, Kitty chunks)
- Export image data to file
- Image deletion/clearing controls
- Performance metrics (load time, decode time)
- Network transfer statistics (for remote sessions)
