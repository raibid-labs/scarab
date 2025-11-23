# Issue #15: Connect Link Hints to Terminal State

**Phase**: 3B - Advanced UI
**Priority**: 🟡 Medium
**Status**: 📝 **Pending**

## 🐛 Problem
The Link Hints UI feature (`crates/scarab-client/src/ui/link_hints.rs`) contains several `TODO` comments indicating it is using hardcoded or dummy data instead of reading from the actual terminal grid.

## 🎯 Goal
Connect the link hint generation logic to the `SharedState` (via `SharedMemoryReader`) to:
1. Scan the actual terminal text for URLs.
2. Calculate screen positions for hints based on the actual grid cells.
3. Implement the "Open" actions (Browser, Editor, Email).

## 🛠 Implementation Details
- **Files**: `crates/scarab-client/src/ui/link_hints.rs`
- **Dependencies**: `SharedState` struct, Rendering grid metrics (for position calculation).

## ✅ Acceptance Criteria
- [ ] `find_links` scans `SharedState.cells` for URL patterns.
- [ ] Hint positions align with the rendered text.
- [ ] Activating a hint triggers `xdg-open` (or platform equivalent).
