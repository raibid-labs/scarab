# Scarab Demo Assets

This directory contains animated GIF demonstrations of Scarab features.

## Demo Files

### Completed Demos
- `link-hints-demo.gif` - Link hints feature demonstration
- `command-palette.gif` - Command palette usage
- `plugin-install.gif` - Plugin installation workflow
- `theme-switch.gif` - Real-time theme switching
- `split-panes.gif` - Split pane navigation (upcoming feature)

### Recording Instructions

All demos are recorded using `asciinema` and converted to GIF using `agg`.

#### Prerequisites

```bash
# Install asciinema (terminal recorder)
pip install asciinema

# Install agg (GIF converter)
cargo install agg

# Or use asciicast2mp4 for videos
npm install -g asciicast2mp4
```

#### Recording Workflow

1. **Record session:**
   ```bash
   asciinema rec demo.cast
   # Perform demo actions
   # Press Ctrl+D when done
   ```

2. **Convert to GIF:**
   ```bash
   # Convert with theme
   agg demo.cast demo.gif --theme dracula

   # Or with custom settings
   agg demo.cast demo.gif \
       --theme monokai \
       --font-family "JetBrains Mono" \
       --font-size 14 \
       --cols 120 \
       --rows 30
   ```

3. **Optimize GIF size:**
   ```bash
   # Install gifsicle
   sudo apt install gifsicle  # Ubuntu/Debian
   brew install gifsicle      # macOS

   # Optimize
   gifsicle -O3 --colors 256 demo.gif -o demo-optimized.gif
   ```

## Demo Scripts

### Link Hints Demo (30 seconds)

```bash
# Script: record-link-hints.sh
asciinema rec link-hints.cast << 'EOF'
# Clear screen
clear
sleep 1

# Show some URLs
echo "Welcome to Scarab Terminal!"
sleep 1
echo ""
echo "Check out these links:"
sleep 1
echo "  - Documentation: https://github.com/raibid-labs/scarab/docs"
sleep 1
echo "  - Issues: https://github.com/raibid-labs/scarab/issues"
sleep 1
echo "  - Website: https://scarab.dev"
sleep 1
echo ""
echo "Press Ctrl+Shift+O to trigger link hints..."
sleep 2

# (User would press Ctrl+Shift+O here - simulate with annotation)
echo "[Link hints activated - press 'a', 'b', or 'c' to open]"
sleep 3

clear
echo "Demo complete!"
sleep 1
EOF

# Convert to GIF
agg link-hints.cast link-hints-demo.gif \
    --theme dracula \
    --font-family "JetBrains Mono" \
    --font-size 14 \
    --cols 80 \
    --rows 24
```

### Command Palette Demo (30 seconds)

```bash
# Script: record-command-palette.sh
asciinema rec command-palette.cast << 'EOF'
clear
sleep 1

echo "Scarab Command Palette Demo"
sleep 1
echo ""
echo "Press Ctrl+Shift+P to open the command palette..."
sleep 2

# (User would press Ctrl+Shift+P - simulate)
echo ""
echo "┌─────────────────────────────────────┐"
echo "│ Command Palette                     │"
echo "├─────────────────────────────────────┤"
echo "│ > sess_                             │"
echo "│                                      │"
echo "│   session new                        │"
echo "│   session list                       │"
echo "│   session switch                     │"
echo "│   session rename                     │"
echo "└─────────────────────────────────────┘"
sleep 3

clear
echo "Demo complete!"
sleep 1
EOF

agg command-palette.cast command-palette.gif \
    --theme monokai \
    --font-size 14 \
    --cols 80 \
    --rows 24
```

### Plugin Install Demo (45 seconds)

```bash
# Script: record-plugin-install.sh
asciinema rec plugin-install.cast << 'EOF'
clear
sleep 1

echo "Installing a Scarab Plugin"
echo "=========================="
sleep 1
echo ""
echo "1. Create plugin directory:"
echo "   $ mkdir -p ~/.config/scarab/plugins"
sleep 2

echo ""
echo "2. Create hello-world plugin:"
echo "   $ vim ~/.config/scarab/plugins/hello.fsx"
sleep 2

echo ""
echo "3. Plugin content:"
cat << 'PLUGIN'
open Scarab.PluginApi

let metadata = {
    Name = "hello-plugin"
    Version = "1.0.0"
    Description = "My first plugin"
    Author = "Your Name"
}

let on_load ctx = async {
    ctx.Log(LogLevel.Info, "Hello from plugin!")
    return Ok ()
}

Plugin.Register { Metadata = metadata; OnLoad = Some on_load }
PLUGIN
sleep 4

echo ""
echo "4. Restart daemon to load plugin"
sleep 2

echo ""
echo "✓ Plugin loaded successfully!"
sleep 2
EOF

agg plugin-install.cast plugin-install.gif \
    --theme nord \
    --font-size 13 \
    --cols 90 \
    --rows 30
```

### Theme Switch Demo (20 seconds)

```bash
# Script: record-theme-switch.sh
asciinema rec theme-switch.cast << 'EOF'
clear
sleep 1

echo "Theme Switching Demo"
echo "==================="
sleep 1
echo ""
echo "Current theme: Dracula"
sleep 2

echo ""
echo "Press Ctrl+Shift+P > theme monokai"
sleep 2

# Simulate theme change
clear
echo "Theme Switching Demo"
echo "==================="
echo ""
echo "Current theme: Monokai"
sleep 2

echo ""
echo "Themes switch instantly - no restart needed!"
sleep 2
EOF

agg theme-switch.cast theme-switch.gif \
    --theme dracula \
    --font-size 14 \
    --cols 70 \
    --rows 20
```

## File Size Guidelines

Keep GIF files under 10MB for GitHub compatibility:

- **Optimal:** < 5MB
- **Maximum:** 10MB
- **Duration:** 15-45 seconds
- **Resolution:** 80x24 to 120x30 columns

If a GIF exceeds 10MB:
1. Reduce color palette (`--colors 128`)
2. Shorten duration
3. Reduce frame rate (agg default is good)
4. Use smaller terminal size

## Attribution

All demos are created with:
- `asciinema` - Terminal session recorder
- `agg` - GIF converter
- Scarab Terminal - The subject of these demos!
