# Issue #14: Integrate Text Rendering Engine in Client

**Phase**: 1B - Core Terminal Emulation
**Priority**: 🔴 Critical
**Status**: ✅ **Fixed**

## 🐛 Problem
The `scarab-client` has a comprehensive text rendering engine implemented in `crates/scarab-client/src/rendering/`, but it is not wired up in `src/main.rs`.
Currently, `main.rs` only prints "Grid updated!" to stdout when the shared memory changes, leaving the graphical window empty (or just a default sprite).

## 🎯 Goal
Update `crates/scarab-client/src/main.rs` to:
1. Import the rendering module.
2. Initialize the `TextRenderer` resource.
3. Add the rendering systems to the Bevy `App`.
4. Replace the `TODO: Update texture/mesh from state.cells` comment in `sync_grid` with the actual update call to the renderer.

## 🛠 Implementation Details
- **Files**: `crates/scarab-client/src/main.rs`
- **Dependencies**: `crates/scarab-client/src/rendering/mod.rs`
- **Reference**: See `docs/text-rendering-implementation.md` for usage details.

## ✅ Acceptance Criteria
- [x] `main.rs` compiles without errors.
- [x] Running the client displays a grid of text (even if empty initially).
- [x] Updating shared memory (via daemon) triggers a visual update in the Bevy window.
- [x] `sync_grid` function calls the rendering mesh generator.

**Resolution**: Replaced `main.rs` content to use `IntegrationPlugin` which properly sets up the text rendering pipeline.
