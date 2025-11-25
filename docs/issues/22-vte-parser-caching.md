# Issue #22: Implement VTE Parser Caching Optimization

**Phase**: 5A - Performance Optimization
**Priority**: 🟢 Low
**Status**: 📝 **Pending**

## 🐛 Problem
The VTE parser optimization module (`scarab-daemon/src/vte_optimized.rs`) contains a TODO comment at line 232:
> "TODO: Implement actual caching logic when integrating this optimization module"

The `SequenceCache` struct exists but has no actual caching implementation. This means repeated VTE sequences are parsed from scratch every time, even though many escape sequences are repetitive (colors, cursor movements, etc.).

## 🎯 Goal
Implement intelligent VTE sequence caching to improve terminal output performance:
1. Cache frequently-used VTE escape sequences and their parsed results
2. Use LRU eviction to prevent unbounded memory growth
3. Add cache hit/miss metrics for monitoring
4. Integrate with the main VTE parsing loop

## 🛠 Implementation Details
- **Files**: `crates/scarab-daemon/src/vte_optimized.rs`
- **Caching Strategy**:
  - Use `lru` crate or custom LRU implementation
  - Cache key: raw byte sequence (e.g., `"\x1b[32m"`)
  - Cache value: parsed `Perform` action
  - Default capacity: 256 entries
  - Cache common sequences: colors, cursor movements, clear operations

- **Key Changes**:
  ```rust
  pub struct SequenceCache {
      cache: LruCache<Vec<u8>, Perform>,
      hits: AtomicU64,
      misses: AtomicU64,
  }

  impl SequenceCache {
      pub fn get(&mut self, sequence: &[u8]) -> Option<&Perform> {
          if let Some(action) = self.cache.get(sequence) {
              self.hits.fetch_add(1, Ordering::Relaxed);
              Some(action)
          } else {
              self.misses.fetch_add(1, Ordering::Relaxed);
              None
          }
      }

      pub fn insert(&mut self, sequence: Vec<u8>, action: Perform) {
          self.cache.put(sequence, action);
      }

      pub fn stats(&self) -> CacheStats { ... }
  }
  ```

## ✅ Acceptance Criteria
- [ ] SequenceCache implements LRU caching
- [ ] Cache is integrated into main VTE parsing flow
- [ ] Cache hit/miss metrics are tracked
- [ ] Performance improvement measurable for repetitive sequences
- [ ] Memory usage remains bounded (via LRU eviction)
- [ ] TODO comment is resolved

## 📋 Testing
- Benchmark parsing performance before/after caching
- Test with output containing many repeated escape sequences (e.g., `ls --color`)
- Verify cache hit rate is >60% for typical terminal usage
- Memory profiling to ensure cache doesn't grow unbounded
- Unit tests for cache eviction behavior

## 🎯 Expected Impact
- **Performance**: 20-40% reduction in VTE parsing CPU time
- **Memory**: ~32KB for default 256-entry cache
- **Benefit**: Most noticeable in colorized output (git, ls, syntax highlighting)
