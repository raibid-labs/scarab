use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;
use crate::ast::Module;

/// Cache for parsed AST modules to avoid re-parsing unchanged files
#[derive(Debug)]
pub struct AstCache {
    cache: HashMap<PathBuf, CacheEntry>,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    module: Module,
    modified_time: SystemTime,
    content_hash: u64,
}

impl AstCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Get a cached module if it exists and is still valid
    pub fn get(&self, path: &PathBuf, modified_time: SystemTime, content_hash: u64) -> Option<&Module> {
        self.cache.get(path).and_then(|entry| {
            if entry.modified_time == modified_time && entry.content_hash == content_hash {
                Some(&entry.module)
            } else {
                None
            }
        })
    }

    /// Insert a parsed module into the cache
    pub fn insert(&mut self, path: PathBuf, module: Module, modified_time: SystemTime, content_hash: u64) {
        self.cache.insert(
            path,
            CacheEntry {
                module,
                modified_time,
                content_hash,
            },
        );
    }

    /// Remove a cached module
    pub fn remove(&mut self, path: &PathBuf) -> Option<Module> {
        self.cache.remove(path).map(|entry| entry.module)
    }

    /// Clear the entire cache
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// Get the number of cached modules
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            entries: self.cache.len(),
            paths: self.cache.keys().cloned().collect(),
        }
    }
}

impl Default for AstCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entries: usize,
    pub paths: Vec<PathBuf>,
}

/// Simple hash function for content
pub fn hash_content(content: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Module;

    #[test]
    fn test_cache_basic() {
        let mut cache = AstCache::new();
        let path = PathBuf::from("test.fsx");
        let module = Module::empty();
        let time = SystemTime::now();
        let hash = 12345;

        cache.insert(path.clone(), module.clone(), time, hash);
        assert_eq!(cache.len(), 1);

        let cached = cache.get(&path, time, hash);
        assert!(cached.is_some());

        // Different hash should miss
        let cached = cache.get(&path, time, 99999);
        assert!(cached.is_none());
    }

    #[test]
    fn test_hash_content() {
        let content1 = "let x = 42";
        let content2 = "let x = 42";
        let content3 = "let y = 99";

        assert_eq!(hash_content(content1), hash_content(content2));
        assert_ne!(hash_content(content1), hash_content(content3));
    }
}
