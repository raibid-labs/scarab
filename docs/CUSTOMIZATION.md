# Scarab Customization Guide

This guide covers how to customize Scarab's appearance and behavior, including the application icon.

## Custom Application Icon

Scarab installs with a default icon, but you can easily replace it with your own.

### Icon Locations

After installation, icons are placed in:
- **SVG**: `~/.local/share/icons/hicolor/scalable/apps/scarab.svg`
- **PNG** (if ImageMagick is installed):
  - `~/.local/share/icons/hicolor/128x128/apps/scarab.png`
  - `~/.local/share/icons/hicolor/64x64/apps/scarab.png`
  - `~/.local/share/icons/hicolor/48x48/apps/scarab.png`
  - `~/.local/share/icons/hicolor/32x32/apps/scarab.png`

### Method 1: Replace the Default Icon

1. **Create your custom icon** (SVG format recommended for best quality):
   ```bash
   # Your custom icon
   my-icon.svg
   ```

2. **Replace the default**:
   ```bash
   # Backup the original
   cp ~/.local/share/icons/hicolor/scalable/apps/scarab.svg ~/.local/share/icons/hicolor/scalable/apps/scarab.svg.bak

   # Replace with your custom icon
   cp my-icon.svg ~/.local/share/icons/hicolor/scalable/apps/scarab.svg
   ```

3. **Generate PNG sizes** (optional, requires ImageMagick):
   ```bash
   convert -background none my-icon.svg -resize 128x128 ~/.local/share/icons/hicolor/128x128/apps/scarab.png
   convert -background none my-icon.svg -resize 64x64 ~/.local/share/icons/hicolor/64x64/apps/scarab.png
   convert -background none my-icon.svg -resize 48x48 ~/.local/share/icons/hicolor/48x48/apps/scarab.png
   convert -background none my-icon.svg -resize 32x32 ~/.local/share/icons/hicolor/32x32/apps/scarab.png
   ```

4. **Update the icon cache**:
   ```bash
   gtk-update-icon-cache -f -t ~/.local/share/icons/hicolor/
   ```

5. **Refresh your desktop environment** (may require logout/login or restart)

### Method 2: Custom Icon Before Installation

If you haven't installed Scarab yet, or want to reinstall with a custom icon:

1. **Replace the source icon**:
   ```bash
   cd /path/to/scarab
   cp my-icon.svg assets/icon.svg
   ```

2. **Install Scarab**:
   ```bash
   just install
   ```

The installation process will automatically use your custom icon.

### Icon Requirements

- **Format**: SVG (recommended) or PNG
- **Size**: 128x128px minimum for PNG
- **Transparency**: Supported (recommended for best appearance)
- **Colors**: Any, but consider visibility on both light and dark backgrounds

### Desktop File Location

The `.desktop` file is located at:
```
~/.local/share/applications/scarab.desktop
```

You can edit this file to customize other application metadata like categories, keywords, or the comment.

## Troubleshooting

### Icon doesn't appear in app menu

1. **Update caches**:
   ```bash
   gtk-update-icon-cache -f -t ~/.local/share/icons/hicolor/
   update-desktop-database ~/.local/share/applications/
   ```

2. **Verify icon paths** in the desktop file:
   ```bash
   cat ~/.local/share/applications/scarab.desktop
   ```
   The `Icon=` line should show `Icon=scarab`

3. **Restart your desktop environment** or log out and back in

4. **Check icon files exist**:
   ```bash
   ls -la ~/.local/share/icons/hicolor/scalable/apps/scarab.svg
   ```

### PNG icons not generated during installation

Install ImageMagick:
```bash
# Ubuntu/Debian
sudo apt install imagemagick

# Fedora
sudo dnf install ImageMagick

# Arch
sudo pacman -S imagemagick

# macOS
brew install imagemagick
```

Then reinstall:
```bash
just install
```

## Other Customization Options

### Terminal Configuration

Edit `~/.config/scarab/config.toml` or `~/.config/scarab/config.fsx` (Fusabi format):

```toml
[terminal]
columns = 120
rows = 40
scrollback_lines = 20000

[font]
family = "JetBrains Mono"
size = 14.0

[colors]
background = "#1a1a1a"
foreground = "#00ff88"

[ui]
cursor_blink = true
animations = true
```

### Keyboard Shortcuts

```toml
[keybindings]
copy_mode = "Ctrl+Shift+C"
paste = "Ctrl+Shift+V"
search = "Ctrl+Shift+F"
command_palette = "Ctrl+Shift+P"
```

### Plugins

Install community plugins or write your own in Fusabi (.fsx):

```bash
just plugin-new my-plugin
just dev-mode my-plugin
```

See [PLUGIN_DEVELOPMENT.md](PLUGIN_DEVELOPMENT.md) for more details.
