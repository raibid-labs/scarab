# Issue #17: Fix Broken Fusabi Dependencies

**Phase**: Maintenance
**Priority**: 🔴 Critical
**Status**: ✅ **Fixed**

## 🐛 Problem
The project fails to build because `fusabi-vm` and `fusabi-frontend` dependencies in `Cargo.toml` point to non-existent local paths (`../../../fusabi-lang/...`).
Code investigation reveals that these dependencies are currently **unused** in the codebase (no `use fusabi` statements).

## 🎯 Goal
Remove the broken dependencies to restore buildability (`cargo check`).
Re-add them when:
1. The `fusabi` code is actually integrated.
2. The dependencies point to a valid location (git repo or published crate), or the local path is corrected.

## 🛠 Implementation Details
- **Files**: `Cargo.toml`, `crates/scarab-client/Cargo.toml`, `crates/scarab-daemon/Cargo.toml`
- **Action**: Comment out or remove the dependency lines.

## ✅ Acceptance Criteria
- [x] `cargo check` passes.
- [x] `cargo build` passes.

**Resolution**: Commented out `fusabi-vm` and `fusabi-frontend` dependencies in all `Cargo.toml` files as they were unused.
