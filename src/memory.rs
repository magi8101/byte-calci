//! Memory Manager - Handles memory allocation for the VM
//!
//! This module provides:
//!   - Arena-based allocation for efficient memory management
//!   - Object tracking for garbage collection
//!   - Memory statistics and monitoring

use std::alloc::{alloc, dealloc, Layout};
use std::cell::Cell;
use std::ptr::NonNull;

/// Memory block header for tracking allocations
#[repr(C)]
struct BlockHeader {
    size: usize,
    marked: Cell<bool>,
    next: Option<NonNull<BlockHeader>>,
}

/// Statistics about memory usage
#[derive(Debug, Clone, Default)]
pub struct MemoryStats {
    pub total_allocated: usize,
    pub total_freed: usize,
    pub current_usage: usize,
    pub peak_usage: usize,
    pub allocation_count: usize,
    pub deallocation_count: usize,
}

impl MemoryStats {
    fn record_allocation(&mut self, size: usize) {
        self.total_allocated += size;
        self.current_usage += size;
        self.allocation_count += 1;
        if self.current_usage > self.peak_usage {
            self.peak_usage = self.current_usage;
        }
    }

    fn record_deallocation(&mut self, size: usize) {
        self.total_freed += size;
        self.current_usage = self.current_usage.saturating_sub(size);
        self.deallocation_count += 1;
    }
}

/// Memory manager with arena-based allocation
pub struct MemoryManager {
    /// Head of the allocation list
    head: Option<NonNull<BlockHeader>>,
    /// Memory statistics
    stats: MemoryStats,
    /// Threshold for triggering GC (in bytes)
    gc_threshold: usize,
    /// Growth factor for GC threshold
    gc_growth_factor: f64,
}

impl MemoryManager {
    /// Create a new memory manager with default settings
    pub fn new() -> Self {
        Self::with_threshold(1024 * 1024) // 1MB default threshold
    }

    /// Create a memory manager with custom GC threshold
    pub fn with_threshold(threshold: usize) -> Self {
        MemoryManager {
            head: None,
            stats: MemoryStats::default(),
            gc_threshold: threshold,
            gc_growth_factor: 2.0,
        }
    }

    /// Allocate memory of given size
    pub fn allocate(&mut self, size: usize) -> Option<NonNull<u8>> {
        let header_size = std::mem::size_of::<BlockHeader>();
        let total_size = header_size + size;
        let align = std::mem::align_of::<BlockHeader>();

        let layout = Layout::from_size_align(total_size, align).ok()?;

        unsafe {
            let ptr = alloc(layout);
            if ptr.is_null() {
                return None;
            }

            // Initialize header
            let header = ptr as *mut BlockHeader;
            (*header).size = size;
            (*header).marked = Cell::new(false);
            (*header).next = self.head;

            // Add to allocation list
            self.head = Some(NonNull::new_unchecked(header));
            self.stats.record_allocation(total_size);

            // Return pointer to data area (after header)
            let data_ptr = ptr.add(header_size);
            Some(NonNull::new_unchecked(data_ptr))
        }
    }

    /// Deallocate a specific block
    unsafe fn deallocate_block(&mut self, header: NonNull<BlockHeader>) {
        let header_size = std::mem::size_of::<BlockHeader>();
        let total_size = header_size + (*header.as_ptr()).size;
        let align = std::mem::align_of::<BlockHeader>();

        let layout = Layout::from_size_align_unchecked(total_size, align);
        dealloc(header.as_ptr() as *mut u8, layout);

        self.stats.record_deallocation(total_size);
    }

    /// Check if GC should be triggered
    pub fn should_collect(&self) -> bool {
        self.stats.current_usage >= self.gc_threshold
    }

    /// Mark a block as reachable
    pub fn mark(&self, ptr: NonNull<u8>) {
        unsafe {
            let header_size = std::mem::size_of::<BlockHeader>();
            let header_ptr = (ptr.as_ptr() as *mut u8).sub(header_size) as *mut BlockHeader;
            (*header_ptr).marked.set(true);
        }
    }

    /// Clear all marks (prepare for marking phase)
    pub fn unmark_all(&mut self) {
        let mut current = self.head;
        while let Some(header) = current {
            unsafe {
                (*header.as_ptr()).marked.set(false);
                current = (*header.as_ptr()).next;
            }
        }
    }

    /// Sweep unmarked objects (deallocation phase)
    pub fn sweep(&mut self) -> usize {
        let mut freed_count = 0;
        let mut prev: Option<NonNull<BlockHeader>> = None;
        let mut current = self.head;

        while let Some(header) = current {
            unsafe {
                let next = (*header.as_ptr()).next;

                if !(*header.as_ptr()).marked.get() {
                    // Remove from list
                    match prev {
                        Some(p) => (*p.as_ptr()).next = next,
                        None => self.head = next,
                    }

                    // Deallocate
                    self.deallocate_block(header);
                    freed_count += 1;
                } else {
                    // Clear mark for next cycle
                    (*header.as_ptr()).marked.set(false);
                    prev = Some(header);
                }

                current = next;
            }
        }

        // Adjust threshold after collection
        if self.stats.current_usage > 0 {
            self.gc_threshold =
                ((self.stats.current_usage as f64) * self.gc_growth_factor) as usize;
        }

        freed_count
    }

    /// Get memory statistics
    pub fn stats(&self) -> &MemoryStats {
        &self.stats
    }

    /// Get current memory usage
    pub fn current_usage(&self) -> usize {
        self.stats.current_usage
    }

    /// Get allocation count
    pub fn allocation_count(&self) -> usize {
        self.stats.allocation_count
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for MemoryManager {
    fn drop(&mut self) {
        // Clean up all remaining allocations
        let mut current = self.head;
        while let Some(header) = current {
            unsafe {
                let next = (*header.as_ptr()).next;
                self.deallocate_block(header);
                current = next;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocate_and_stats() {
        let mut mm = MemoryManager::new();
        let _ptr = mm.allocate(64).expect("Allocation failed");

        assert!(mm.stats().current_usage > 0);
        assert_eq!(mm.stats().allocation_count, 1);
    }

    #[test]
    fn test_mark_and_sweep() {
        let mut mm = MemoryManager::new();
        let ptr1 = mm.allocate(64).expect("Allocation failed");
        let _ptr2 = mm.allocate(64).expect("Allocation failed");

        assert_eq!(mm.stats().allocation_count, 2);

        // Mark only ptr1
        mm.unmark_all();
        mm.mark(ptr1);

        // Sweep should free ptr2
        let freed = mm.sweep();
        assert_eq!(freed, 1);
        assert_eq!(mm.stats().deallocation_count, 1);
    }
}
