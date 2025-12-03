# Focusable Detection Audit Report

**Date**: 2025-12-03
**Issue**: #40 - Navigation: Audit and unify focusable detection paths
**Auditor**: Claude Code (Rust Expert)

## Executive Summary

This audit identified **three independent regex-based detection systems** operating in parallel across the Scarab codebase, resulting in:
- **Duplicated regex patterns** (URL, filepath, email detection)
- **Inconsistent validation logic** across systems
- **Performance overhead** from multiple scans of the same terminal content
- **Maintenance burden** with three separate codebases to update

**Recommendation**: Consolidate all detection logic into `FocusableDetector` as the single source of truth, with other systems delegating to it or using ECS events.

---

## Detailed Findings

### 1. FocusableDetector (Primary Detection System)
**Location**: `crates/scarab-client/src/navigation/focusable.rs`
**Status**: Modern ECS-based system, production-ready

#### Regex Patterns
```rust
// Lines 139-146
url_regex: r"https?://[^\s<>{}|\^~\[\]`]+|www\.[^\s<>{}|\^~\[\]`]+"
filepath_regex: r"(?:~|\.{1,2}|/)?(?:[a-zA-Z0-9_\-./]+/)*[a-zA-Z0-9_\-.]+\.[a-zA-Z]{2,5}"
email_regex: r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}"
```

#### Features
- Compiled regex caching in `FocusableDetector` resource
- Configurable via `FocusableScanConfig`
- Max focusables limit (500 default)
- Additional validation for file paths (length > 3)
- Grid coordinate tracking
- ECS event-driven architecture

#### Architecture
- Scans on `EnterHintModeEvent`
- Creates `FocusableRegion` entities
- Converts grid coords to world coords
- Integrates with prompt markers
- Zone filtering support

---

### 2. LinkDetector (Legacy Vimium-Style System)
**Location**: `crates/scarab-client/src/ui/link_hints.rs`
**Status**: Legacy system, overlaps with FocusableDetector

#### Regex Patterns
```rust
// Lines 63-69
url_regex: r"https?://[^\s<>{}|\^~\[\]`]+|www\.[^\s<>{}|\^~\[\]`]+"
filepath_regex: r"(?:~|\.{1,2}|/)?(?:[a-zA-Z0-9_\-./]+/)*[a-zA-Z0-9_\-.]+"
email_regex: r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}"
```

#### Differences from FocusableDetector
- **URL regex**: Identical
- **Filepath regex**:
  - FocusableDetector requires file extension: `\.[a-zA-Z]{2,5}` (more restrictive)
  - LinkDetector allows no extension: ends with `+` (less restrictive)
  - LinkDetector adds validation: `path.len() > 3 && (path.contains('/') || path.contains('.'))`
- **Email regex**: Identical

#### Features
- Activated by `Ctrl+K` (hardcoded)
- Creates `LinkHint` components (not `FocusableRegion`)
- Hint key generation (`a`, `b`, ..., `aa`, `ab`, ...)
- Direct browser/editor launching
- No ECS event integration

#### Issues Identified
1. **Duplication**: Entire detection pipeline reimplemented
2. **Inconsistency**: Different filepath validation logic
3. **Isolation**: Does not participate in ECS navigation events
4. **Trigger Conflict**: `Ctrl+K` may conflict with other keybindings

---

### 3. NavigationPlugin (scarab-nav)
**Location**: `crates/scarab-nav/src/lib.rs`
**Status**: Plugin system, should use ECS not internal regex

#### Regex Patterns
```rust
// Line 100
url_regex: r"https?://[^\s\)]+".unwrap()
```

#### Features
- Activated by `Alt+f` or `Ctrl+f`
- Listens for protocol layout updates via Unix socket
- Augments protocol with regex URL detection
- Generates hint labels using custom charset: `"asdfghjklqwertyuiopzxcvbnm"`
- Sends overlay draw commands to daemon

#### Issues Identified
1. **Incomplete Pattern**: Only detects URLs, missing filepaths and emails
2. **Inconsistent Pattern**: Different URL regex (allows `)` inside URL, excludes other chars)
3. **Architecture Mismatch**: Plugin should consume ECS events from FocusableDetector, not scan independently
4. **Coupling**: Direct terminal content scanning via `ctx.get_line(y)` couples plugin to daemon internals

---

## Pattern Comparison Matrix

| Pattern Type | FocusableDetector | LinkDetector | NavigationPlugin |
|--------------|-------------------|--------------|------------------|
| **URL** | `https?://[^\s<>{}|\^~\[\]` `]+\|www\.[^\s<>{}|\^~\[\]` `]+` | Same | `https?://[^\s\)]+` |
| **Filepath** | `(?:~\|\.{1,2}\|/)?(?:[a-zA-Z0-9_\-./]+/)*[a-zA-Z0-9_\-.]+\.[a-zA-Z]{2,5}` | `(?:~\|\.{1,2}\|/)?(?:[a-zA-Z0-9_\-./]+/)*[a-zA-Z0-9_\-.]+` | N/A |
| **Email** | `[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}` | Same | N/A |
| **Validation** | `path.len() > 3` | `path.len() > 3 && (contains('/') \|\| contains('.'))` | None |

---

## Missing Features

### FocusableSource::Image
**Status**: Missing
**Requirement**: Add support for detecting images in terminal (Sixel, iTerm2, Kitty protocols)

Current `FocusableSource` enum:
```rust
pub enum FocusableSource {
    Terminal,
    Ratatui,
    PromptMarker,
}
```

**Recommendation**: Add `Image` variant to support future image protocol navigation.

---

## Recommended Unification Strategy

### Phase 1: Consolidate Detection Logic
1. **Make FocusableDetector the canonical detector**
   - Keep existing patterns (most restrictive filepath for safety)
   - Add configuration for pattern customization
   - Add `FocusableSource::Image` variant

2. **Refactor LinkDetector**
   - Remove duplicate regex patterns
   - Delegate detection to `FocusableDetector`
   - Keep hint key generation and UI logic
   - Coordinate with FocusablePlugin to avoid conflicts

3. **Update NavigationPlugin**
   - Remove internal URL regex
   - Subscribe to `FocusableRegion` ECS entities via events
   - Use protocol layout as primary source
   - Fall back to ECS focusables when protocol unavailable

### Phase 2: Integration
1. **Emit FocusableScannedEvent**
   - Create event when `FocusableDetector` completes scan
   - Include detected count, pattern config used
   - Allow downstream systems to react

2. **Shared Hint Labeling**
   - Extract hint key generation into utility module
   - Support different label schemes (alphabetic, home row, dvorak)
   - Configurable via resource

3. **Unified Activation**
   - Single keybinding for hint mode (recommend `Ctrl+;`)
   - Mode-specific behavior via NavMode enum
   - Deprecate hardcoded `Ctrl+K` in LinkDetector

### Phase 3: Testing
1. **Add regex pattern tests**
   - Verify all three patterns against common test cases
   - Edge cases: URLs with Unicode, nested paths, etc.

2. **Add integration tests**
   - Verify LinkDetector delegates correctly
   - Verify NavigationPlugin receives ECS events
   - Verify no duplicate detection work

---

## Performance Impact Analysis

### Current State (Duplicated)
When hint mode activates:
1. FocusableDetector scans terminal (O(n) lines × O(m) patterns)
2. LinkDetector scans terminal (O(n) lines × O(m) patterns)
3. NavigationPlugin scans terminal (O(n) lines × 1 pattern)

**Total**: 3x full terminal scans

### Proposed State (Unified)
When hint mode activates:
1. FocusableDetector scans terminal once (O(n) lines × O(m) patterns)
2. LinkDetector reads `FocusableRegion` entities (O(k) focusables, no regex)
3. NavigationPlugin reads `FocusableRegion` entities (O(k) focusables, no regex)

**Total**: 1x full terminal scan + 2x lightweight ECS queries

**Estimated Performance Gain**: 60-70% reduction in hint mode activation latency for large terminals (200×100 grid).

---

## Implementation Checklist

- [ ] Add `FocusableSource::Image` variant
- [ ] Add `FocusableScannedEvent` to navigation module
- [ ] Refactor `LinkDetector` to delegate to `FocusableDetector`
- [ ] Update `NavigationPlugin` to consume ECS events
- [ ] Extract hint key generation into shared utility
- [ ] Add deprecation comments for old code paths
- [ ] Add integration tests for unified detection
- [ ] Update documentation to reference canonical detector
- [ ] Benchmark performance improvements

---

## Migration Path

### Immediate (This Commit)
1. Add `FocusableSource::Image`
2. Add delegation path in `LinkDetector`
3. Add deprecation warnings to `NavigationPlugin` regex

### Short-term (Next Sprint)
1. Remove regex from `NavigationPlugin`
2. Implement ECS event subscription
3. Coordinate keybinding unification

### Long-term (Future)
1. Remove `LinkHintsPlugin` entirely (merge into FocusablePlugin)
2. Consolidate hint rendering systems
3. Add image protocol detection

---

## Conclusion

The current architecture has **three independent detection systems** that evolved separately but serve overlapping purposes. This audit recommends consolidating all detection logic into the modern ECS-based `FocusableDetector` system, which already has the most complete implementation.

**Benefits of unification**:
- Single source of truth for regex patterns
- Consistent behavior across all navigation modes
- 60-70% performance improvement on hint activation
- Reduced maintenance burden
- Better testability

**Implementation effort**: Medium (estimated 8-12 hours)
**Risk level**: Low (backward compatible via delegation)
