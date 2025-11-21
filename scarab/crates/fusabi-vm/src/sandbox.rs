//! Memory Sandbox for VM Security
//!
//! Enforces memory limits and prevents unsafe operations

use std::alloc::{alloc, dealloc, Layout};
use std::ptr::NonNull;

/// Default memory limit: 1GB
pub const DEFAULT_MEMORY_LIMIT: usize = 1024 * 1024 * 1024;

/// Maximum allocation size
pub const MAX_ALLOCATION_SIZE: usize = 100 * 1024 * 1024; // 100MB

/// Memory sandbox
pub struct Sandbox {
    /// Total memory limit
    memory_limit: usize,

    /// Current memory usage
    memory_used: usize,

    /// Allocated blocks
    allocations: Vec<AllocationInfo>,
}

#[derive(Debug, Clone)]
struct AllocationInfo {
    ptr: usize,
    size: usize,
    layout: Layout,
}

impl Sandbox {
    /// Create a new sandbox with default limits
    pub fn new() -> Self {
        Self::with_limit(DEFAULT_MEMORY_LIMIT)
    }

    /// Create a sandbox with custom memory limit
    pub fn with_limit(limit: usize) -> Self {
        Self {
            memory_limit: limit,
            memory_used: 0,
            allocations: Vec::new(),
        }
    }

    /// Allocate memory within sandbox
    pub fn allocate(&mut self, size: usize) -> Result<NonNull<u8>, SandboxError> {
        // Check size limit
        if size > MAX_ALLOCATION_SIZE {
            return Err(SandboxError::AllocationTooLarge(size));
        }

        // Check if allocation would exceed limit
        if self.memory_used + size > self.memory_limit {
            return Err(SandboxError::MemoryLimitExceeded {
                requested: size,
                available: self.memory_limit - self.memory_used,
            });
        }

        // Create layout
        let layout = Layout::from_size_align(size, 8)
            .map_err(|_| SandboxError::InvalidLayout)?;

        // Allocate
        let ptr = unsafe { alloc(layout) };
        if ptr.is_null() {
            return Err(SandboxError::AllocationFailed);
        }

        let ptr = NonNull::new(ptr).expect("Allocated pointer should not be null");

        // Track allocation
        self.allocations.push(AllocationInfo {
            ptr: ptr.as_ptr() as usize,
            size,
            layout,
        });

        self.memory_used += size;

        Ok(ptr)
    }

    /// Deallocate memory
    pub fn deallocate(&mut self, ptr: NonNull<u8>) -> Result<(), SandboxError> {
        let ptr_addr = ptr.as_ptr() as usize;

        // Find allocation
        let idx = self
            .allocations
            .iter()
            .position(|a| a.ptr == ptr_addr)
            .ok_or(SandboxError::InvalidPointer)?;

        let info = self.allocations.remove(idx);

        // Deallocate
        unsafe {
            dealloc(ptr.as_ptr(), info.layout);
        }

        self.memory_used -= info.size;

        Ok(())
    }

    /// Check if a memory access is valid
    pub fn validate_access(&self, ptr: usize, size: usize) -> Result<(), SandboxError> {
        // Find containing allocation
        let _allocation = self
            .allocations
            .iter()
            .find(|a| ptr >= a.ptr && ptr + size <= a.ptr + a.size)
            .ok_or(SandboxError::InvalidMemoryAccess { ptr, size })?;

        Ok(())
    }

    /// Get current memory usage
    pub fn memory_used(&self) -> usize {
        self.memory_used
    }

    /// Get memory limit
    pub fn memory_limit(&self) -> usize {
        self.memory_limit
    }

    /// Get number of allocations
    pub fn allocation_count(&self) -> usize {
        self.allocations.len()
    }

    /// Reset sandbox (deallocate all memory)
    pub fn reset(&mut self) {
        // Deallocate all blocks
        for info in self.allocations.drain(..) {
            unsafe {
                dealloc(info.ptr as *mut u8, info.layout);
            }
        }

        self.memory_used = 0;
    }

    /// Check if sandbox is within limits
    pub fn check_limits(&self) -> Result<(), SandboxError> {
        if self.memory_used > self.memory_limit {
            return Err(SandboxError::MemoryLimitExceeded {
                requested: 0,
                available: 0,
            });
        }

        Ok(())
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        self.reset();
    }
}

impl Default for Sandbox {
    fn default() -> Self {
        Self::new()
    }
}

/// Sandbox Errors
#[derive(Debug, thiserror::Error)]
pub enum SandboxError {
    #[error("Memory limit exceeded: requested {requested}, available {available}")]
    MemoryLimitExceeded { requested: usize, available: usize },

    #[error("Allocation too large: {0} bytes (max {MAX_ALLOCATION_SIZE})")]
    AllocationTooLarge(usize),

    #[error("Allocation failed")]
    AllocationFailed,

    #[error("Invalid memory layout")]
    InvalidLayout,

    #[error("Invalid pointer")]
    InvalidPointer,

    #[error("Invalid memory access at 0x{ptr:x}, size {size}")]
    InvalidMemoryAccess { ptr: usize, size: usize },

    #[error("Syscall not allowed: {0}")]
    SyscallDenied(String),
}

/// Security policy
pub struct SecurityPolicy {
    /// Allow file I/O
    pub allow_file_io: bool,

    /// Allow network access
    pub allow_network: bool,

    /// Allow spawning processes
    pub allow_process_spawn: bool,

    /// Allow environment variable access
    pub allow_env_access: bool,
}

impl SecurityPolicy {
    /// Create a restrictive policy (no permissions)
    pub fn restrictive() -> Self {
        Self {
            allow_file_io: false,
            allow_network: false,
            allow_process_spawn: false,
            allow_env_access: false,
        }
    }

    /// Create a permissive policy (all permissions)
    pub fn permissive() -> Self {
        Self {
            allow_file_io: true,
            allow_network: true,
            allow_process_spawn: true,
            allow_env_access: true,
        }
    }

    /// Check if an operation is allowed
    pub fn check_permission(&self, operation: &str) -> Result<(), SandboxError> {
        let allowed = match operation {
            "file_read" | "file_write" => self.allow_file_io,
            "network_connect" | "network_bind" => self.allow_network,
            "process_spawn" => self.allow_process_spawn,
            "env_read" | "env_write" => self.allow_env_access,
            _ => false,
        };

        if !allowed {
            return Err(SandboxError::SyscallDenied(operation.to_string()));
        }

        Ok(())
    }
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self::restrictive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_creation() {
        let sandbox = Sandbox::new();
        assert_eq!(sandbox.memory_used(), 0);
        assert_eq!(sandbox.memory_limit(), DEFAULT_MEMORY_LIMIT);
    }

    #[test]
    fn test_allocation() {
        let mut sandbox = Sandbox::new();
        let ptr = sandbox.allocate(1024).unwrap();
        assert_eq!(sandbox.memory_used(), 1024);
        assert_eq!(sandbox.allocation_count(), 1);

        sandbox.deallocate(ptr).unwrap();
        assert_eq!(sandbox.memory_used(), 0);
        assert_eq!(sandbox.allocation_count(), 0);
    }

    #[test]
    fn test_memory_limit() {
        let mut sandbox = Sandbox::with_limit(1024);
        let result = sandbox.allocate(2048);
        assert!(result.is_err());
        match result.unwrap_err() {
            SandboxError::MemoryLimitExceeded { .. } => {}
            _ => panic!("Expected MemoryLimitExceeded error"),
        }
    }

    #[test]
    fn test_allocation_too_large() {
        let mut sandbox = Sandbox::new();
        let result = sandbox.allocate(MAX_ALLOCATION_SIZE + 1);
        assert!(result.is_err());
        match result.unwrap_err() {
            SandboxError::AllocationTooLarge(_) => {}
            _ => panic!("Expected AllocationTooLarge error"),
        }
    }

    #[test]
    fn test_reset() {
        let mut sandbox = Sandbox::new();
        sandbox.allocate(1024).unwrap();
        sandbox.allocate(2048).unwrap();

        assert_eq!(sandbox.memory_used(), 3072);
        assert_eq!(sandbox.allocation_count(), 2);

        sandbox.reset();

        assert_eq!(sandbox.memory_used(), 0);
        assert_eq!(sandbox.allocation_count(), 0);
    }

    #[test]
    fn test_security_policy() {
        let policy = SecurityPolicy::restrictive();
        assert!(policy.check_permission("file_read").is_err());
        assert!(policy.check_permission("network_connect").is_err());

        let policy = SecurityPolicy::permissive();
        assert!(policy.check_permission("file_read").is_ok());
        assert!(policy.check_permission("network_connect").is_ok());
    }
}
