# Accessibility Module

Comprehensive accessibility support for Scarab terminal emulator, implementing best practices for assistive technology integration.

## Features

### 1. Export Capabilities
Export terminal content to multiple formats for accessibility and sharing:

- **Plain Text** (`.txt`) - ANSI codes stripped, pure text content
- **HTML** (`.html`) - Preserves colors as CSS, supports styling
- **Markdown** (`.md`) - Code block format for documentation

### 2. Screen Reader Integration (Stubs)
Foundation for future AT-SPI (Assistive Technology Service Provider Interface) integration:

- Announcement events with priority levels
- Cursor movement tracking
- Content change notifications
- Support for Orca and other Linux screen readers (future implementation)

### 3. Visual Accessibility
- **High Contrast Mode** - Enhanced visibility with pure white/black contrast
- **Text Scaling** - Dynamic font size adjustment (0.5x - 3.0x)
- **Keyboard Navigation** - Enhanced keyboard-only operation

## Usage

### Commands

All accessibility commands use the `:a11y` prefix:

```
# Export commands
:a11y export text /tmp/output.txt        # Export to plain text
:a11y export html /tmp/output.html       # Export to HTML with colors
:a11y export markdown /tmp/output.md     # Export to Markdown

# Visual accessibility
:a11y contrast toggle                     # Toggle high contrast mode
:a11y scale 1.5                          # Set text scale to 150%
:a11y scale increase 0.1                 # Increase scale by 10%
:a11y scale decrease 0.1                 # Decrease scale by 10%
:a11y scale reset                        # Reset to 100%

# Help
:a11y help                               # Show all commands
```

### Bevy Integration

The accessibility features are integrated via `AccessibilityPlugin`:

```rust
use scarab_client::AccessibilityPlugin;

app.add_plugins(AccessibilityPlugin);
```

### Event System

Send accessibility events from your code:

```rust
use scarab_client::{ExportGridEvent, ExportFormat, ToggleHighContrastEvent};

// Trigger an export
events.send(ExportGridEvent {
    format: ExportFormat::Html,
    path: "/tmp/terminal.html".to_string(),
});

// Toggle high contrast
events.send(ToggleHighContrastEvent);
```

### Configuration

Accessibility settings are stored in `AccessibilityConfig`:

```rust
#[derive(Resource)]
pub struct AccessibilityConfig {
    pub enabled: bool,                    // Global toggle
    pub high_contrast: bool,              // High contrast mode
    pub announce_output: bool,            // Screen reader announcements
    pub text_scale: f32,                  // Font size multiplier
    pub default_export_format: ExportFormat,
}
```

## Architecture

### Module Structure

```
accessibility/
├── mod.rs              # Main plugin and Bevy integration
├── export.rs           # Export functionality (text/HTML/Markdown)
├── screen_reader.rs    # AT-SPI integration stubs
├── settings.rs         # Configuration and events
└── README.md           # This file
```

### Export Pipeline

1. **Text Export** - Strips ANSI codes, extracts raw character data
2. **HTML Export** - Preserves colors as inline CSS, handles special characters
3. **Markdown Export** - Wraps content in code blocks with header

### Screen Reader Integration (Future)

Current implementation provides event stubs for future AT-SPI integration:

```rust
pub struct AtSpiIntegration {
    // Future: D-Bus connection, accessibility object tree
}

impl AtSpiIntegration {
    pub fn announce(&self, announcement: &Announcement) {
        // TODO: Send to AT-SPI via D-Bus
    }
}
```

Full implementation would require:
- D-Bus bindings (`zbus` crate)
- AT-SPI protocol implementation
- Accessible object hierarchy
- Text interface (`org.a11y.atspi.Text`)
- Value interface for progress indicators

## Testing

Run accessibility tests:

```bash
cargo test -p scarab-client accessibility
```

Test coverage includes:
- Export format conversions
- Command parsing
- Configuration management
- Event handling

## Future Enhancements

### Phase 1: Core Accessibility (Current)
- ✅ Export to text/HTML/Markdown
- ✅ High contrast mode toggle
- ✅ Text scaling support
- ✅ Event infrastructure

### Phase 2: Screen Reader Integration
- [ ] AT-SPI D-Bus connection
- [ ] Implement `org.a11y.atspi.Accessible` interface
- [ ] Implement `org.a11y.atspi.Text` interface
- [ ] Cursor position announcements
- [ ] Content change notifications

### Phase 3: Advanced Features
- [ ] Focus indicators for keyboard navigation
- [ ] Customizable color schemes
- [ ] Voice control integration
- [ ] Braille display support

### Phase 4: Compliance
- [ ] WCAG 2.1 Level AA compliance
- [ ] Section 508 compliance
- [ ] Screen reader testing (Orca, NVDA, JAWS)

## Contributing

When adding accessibility features:

1. **Follow WCAG Guidelines** - Ensure features meet WCAG 2.1 standards
2. **Test with Screen Readers** - Verify with Orca on Linux
3. **Document Thoroughly** - Update this README and code comments
4. **Add Tests** - Include unit tests for new functionality

## Resources

- [WCAG 2.1 Guidelines](https://www.w3.org/WAI/WCAG21/quickref/)
- [AT-SPI Documentation](https://www.freedesktop.org/wiki/Accessibility/AT-SPI2/)
- [Orca Screen Reader](https://wiki.gnome.org/Projects/Orca)
- [Terminal Accessibility Best Practices](https://www.w3.org/WAI/ARIA/apg/patterns/terminal/)

## License

Part of the Scarab terminal emulator project.
