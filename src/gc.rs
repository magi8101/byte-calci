//! Garbage Collector - Mark and Sweep implementation
//!
//! The GC operates in two phases:
//!   1. Mark phase: Traverse from roots and mark reachable objects
//!   2. Sweep phase: Free all unmarked objects
//!
//! For this calculator VM, roots are:
//!   - Values on the VM stack
//!   - Constants in the bytecode chunk

use crate::memory::MemoryManager;
use std::ptr::NonNull;

/// Trait for objects that can be traced by the GC
pub trait Traceable {
    /// Visit all references held by this object
    fn trace(&self, gc: &mut GarbageCollector);
}

/// GC statistics
#[derive(Debug, Clone, Default)]
pub struct GcStats {
    pub collections: usize,
    pub total_objects_freed: usize,
    pub total_bytes_freed: usize,
}

/// Mark-and-sweep garbage collector
pub struct GarbageCollector {
    memory: MemoryManager,
    stats: GcStats,
    /// Roots that should not be collected
    roots: Vec<NonNull<u8>>,
    /// Whether GC is currently running (prevents recursive collection)
    collecting: bool,
}

impl GarbageCollector {
    pub fn new() -> Self {
        GarbageCollector {
            memory: MemoryManager::new(),
            stats: GcStats::default(),
            roots: Vec::new(),
            collecting: false,
        }
    }

    /// Create GC with custom memory threshold
    pub fn with_threshold(threshold: usize) -> Self {
        GarbageCollector {
            memory: MemoryManager::with_threshold(threshold),
            stats: GcStats::default(),
            roots: Vec::new(),
            collecting: false,
        }
    }

    /// Allocate memory, potentially triggering GC
    pub fn allocate(&mut self, size: usize) -> Option<NonNull<u8>> {
        // Check if we should collect before allocating
        if self.should_collect() {
            self.collect();
        }

        self.memory.allocate(size)
    }

    /// Add a root reference
    pub fn add_root(&mut self, ptr: NonNull<u8>) {
        if !self.roots.contains(&ptr) {
            self.roots.push(ptr);
        }
    }

    /// Remove a root reference
    pub fn remove_root(&mut self, ptr: NonNull<u8>) {
        self.roots.retain(|&p| p != ptr);
    }

    /// Clear all roots
    pub fn clear_roots(&mut self) {
        self.roots.clear();
    }

    /// Set roots from a slice of pointers
    pub fn set_roots(&mut self, roots: &[NonNull<u8>]) {
        self.roots.clear();
        self.roots.extend_from_slice(roots);
    }

    /// Check if GC should run
    pub fn should_collect(&self) -> bool {
        !self.collecting && self.memory.should_collect()
    }

    /// Run garbage collection
    pub fn collect(&mut self) -> usize {
        if self.collecting {
            return 0;
        }

        self.collecting = true;
        let bytes_before = self.memory.current_usage();

        // Mark phase
        self.mark_phase();

        // Sweep phase
        let objects_freed = self.sweep_phase();

        let bytes_after = self.memory.current_usage();
        let bytes_freed = bytes_before.saturating_sub(bytes_after);

        self.stats.collections += 1;
        self.stats.total_objects_freed += objects_freed;
        self.stats.total_bytes_freed += bytes_freed;

        self.collecting = false;
        objects_freed
    }

    /// Mark phase: mark all reachable objects starting from roots
    fn mark_phase(&mut self) {
        // Clear all marks
        self.memory.unmark_all();

        // Mark from roots
        for &root in &self.roots {
            self.memory.mark(root);
        }
    }

    /// Sweep phase: free all unmarked objects
    fn sweep_phase(&mut self) -> usize {
        self.memory.sweep()
    }

    /// Force a full garbage collection
    pub fn force_collect(&mut self) -> usize {
        let was_collecting = self.collecting;
        self.collecting = false;
        let result = self.collect();
        self.collecting = was_collecting;
        result
    }

    /// Get GC statistics
    pub fn stats(&self) -> &GcStats {
        &self.stats
    }

    /// Get memory statistics
    pub fn memory_stats(&self) -> &crate::memory::MemoryStats {
        self.memory.stats()
    }

    /// Get current memory usage
    pub fn current_usage(&self) -> usize {
        self.memory.current_usage()
    }
}

impl Default for GarbageCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// A GC-managed value wrapper
#[derive(Debug)]
pub struct GcValue<T> {
    ptr: NonNull<T>,
}

impl<T> GcValue<T> {
    /// Create a new GC-managed value
    pub fn new(gc: &mut GarbageCollector, value: T) -> Option<Self> {
        let size = std::mem::size_of::<T>();
        let ptr = gc.allocate(size)?;

        unsafe {
            let typed_ptr = ptr.as_ptr() as *mut T;
            std::ptr::write(typed_ptr, value);
            Some(GcValue {
                ptr: NonNull::new_unchecked(typed_ptr),
            })
        }
    }

    /// Get a reference to the value
    pub fn get(&self) -> &T {
        unsafe { self.ptr.as_ref() }
    }

    /// Get a mutable reference to the value
    pub fn get_mut(&mut self) -> &mut T {
        unsafe { self.ptr.as_mut() }
    }

    /// Get the raw pointer for rooting
    pub fn as_ptr(&self) -> NonNull<u8> {
        self.ptr.cast()
    }
}

impl<T: Clone> Clone for GcValue<T> {
    fn clone(&self) -> Self {
        // Note: This creates a shallow clone of the pointer, not the value
        // For deep clones, use GcValue::new with the cloned value
        GcValue { ptr: self.ptr }
    }
}

impl<T: Copy> Copy for GcValue<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gc_allocate() {
        let mut gc = GarbageCollector::new();
        let ptr = gc.allocate(64).expect("Allocation failed");
        gc.add_root(ptr);

        // Collection should not free rooted object
        let freed = gc.collect();
        assert_eq!(freed, 0);
    }

    #[test]
    fn test_gc_collect_unreachable() {
        let mut gc = GarbageCollector::with_threshold(1); // Low threshold to trigger GC
        let _ptr = gc.allocate(64).expect("Allocation failed");

        // No roots, so allocation should be freed
        let freed = gc.force_collect();
        assert_eq!(freed, 1);
    }

    #[test]
    fn test_gc_value() {
        let mut gc = GarbageCollector::new();
        let value = GcValue::new(&mut gc, 42.0f64).expect("Allocation failed");
        gc.add_root(value.as_ptr());

        assert_eq!(*value.get(), 42.0);
    }
}
