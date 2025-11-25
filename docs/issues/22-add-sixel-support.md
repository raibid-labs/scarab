# Issue: Feature Gap - Sixel / Image Protocol Support

**Priority:** High (for Daily Driver status)
**Component:** scarab-daemon / scarab-client
**Status:** Open

## Description
Modern terminal workflows often rely on image previews (e.g., `lsix`, `chafa`, `imgcat`). Analysis of `scarab-daemon` and `scarab-client` reveals no implementation of Sixel or iTerm2 image protocols.

## Proposed Fix
1. **Daemon:** Implement Sixel parsing in the VTE layer (or enable it if using `alacritty_terminal`'s parser, ensuring the events are propagated).
2. **Protocol:** Add `Image` variant to `Cell` or `ControlMessage` in `scarab-protocol`.
3. **Client:** Implement a texture generation system in Bevy to render these image blobs at the correct grid coordinates.
