# Issue: Architecture - Redundancy between scarab-nav and scarab-client

**Priority:** Medium
**Component:** scarab-nav / scarab-client
**Status:** Open

## Description
The repository contains a crate `crates/scarab-nav` which implements basic link detection. However, `crates/scarab-client/src/ui/link_hints.rs` implements a more advanced version of the same functionality locally.

## Impact
- Duplicate code / maintenance burden.
- Confusion for contributors regarding where navigation logic resides.

## Proposed Fix
1. Evaluate if `scarab-nav` is intended to be a headless logic crate.
2. If so, move the regex and detection logic from `scarab-client` to `scarab-nav`.
3. If `scarab-nav` is deprecated, remove the crate entirely.
