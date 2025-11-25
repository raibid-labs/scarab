# Demo GIF Placeholders

The actual GIF demos need to be recorded using the scripts in `/home/beengud/raibid-labs/scarab/scripts/`.

## To Record Demos

```bash
cd /home/beengud/raibid-labs/scarab

# Make recording script executable
chmod +x scripts/record-demos.sh

# Install prerequisites
pip install asciinema
cargo install agg

# Run the recording script
./scripts/record-demos.sh
```

This will create:
- `link-hints-demo.gif`
- `command-palette.gif`
- `plugin-install.gif`
- `theme-switch.gif`
- `split-panes.gif`

## Expected Demo Files

The following files should be created after running the script:

1. **link-hints-demo.gif** (~2-5 MB)
   - Shows URL detection and keyboard-driven link opening
   - Duration: ~30 seconds

2. **command-palette.gif** (~2-5 MB)
   - Shows command palette with fuzzy search
   - Duration: ~30 seconds

3. **plugin-install.gif** (~3-6 MB)
   - Shows plugin creation workflow
   - Duration: ~45 seconds

4. **theme-switch.gif** (~1-3 MB)
   - Shows instant theme switching
   - Duration: ~20 seconds

5. **split-panes.gif** (~2-5 MB)
   - Placeholder for upcoming feature
   - Duration: ~30 seconds

## Notes

- All GIFs should be under 10MB for GitHub compatibility
- Use `gifsicle -O3 --colors 256 input.gif -o output.gif` to optimize
- Recommended terminal size: 100x30 columns
- Font: JetBrains Mono 14pt
- Theme: Dracula
