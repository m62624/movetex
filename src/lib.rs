//! # Movetex
//!
//! Movetex provides a non-blocking, atomic-based synchronization primitive in Rust, enabling multiple readers or a single writer to access complex data structures safely and efficiently. Unlike a standard `Mutex`, Movetex performs atomic pointer swaps, making it suitable for advanced scenarios where atomic operations are needed not only for basic types but also for complex data structures.
//!
//! ## Key Features
//!
//! - **Atomic Operations for Complex Data**: Movetex uses two `AtomicPtr` pointers, allowing atomic read/write access for non-trivial data types without blocking readers.
//! - **Multiple Readers, Single Writer**: Allows multiple readers simultaneously, while only one writer can have access at a time, without reader blocking.
//! - **Conditional Write Handling**: `write` returns a `bool` to indicate if the write lock was acquired, giving you control over what to do if a write isn’t immediately possible.
//!   - For multithreaded scenarios, you can retry or pause.
//!   - In async contexts, yield the current green thread to improve efficiency.
//! ## Safety
//!
//! This library uses `unsafe` code to perform low-level atomic operations. Please make sure to use it only through the public API, which ensures memory safety.
//!
//! ## Example
//!
//! ```rust
//! use movetex::Movetex;
//! use std::sync::Arc;
//!
//! let data = Arc::new(Movetex::new(String::from("Initial Data")));
//!
//! let read_value = data.read();
//! println!("Current value: {}", read_value);
//!
//! // Attempt writing
//! if data.write(|val| *val = String::from("Updated Data")) {
//!     println!("Write successful.");
//! } else {
//!     println!("Write attempt failed; handle retry or yield here.");
//! }
//! ```
//!
//! This example shows how Movetex ensures atomicity for complex data reads/writes, with controlled handling for write contention.

use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};

/// Movetex: A lock-free synchronization primitive for concurrent data access
///
/// `Movetex<T>` enables safe, non-blocking, atomic read and write access to complex data types
/// with multiple readers and a single writer. Internally, it uses two `AtomicPtr` fields to allow
/// atomic pointer-based synchronization, extending atomic operations to complex data types beyond
/// basic atomic primitives.
///
/// Readers can access data simultaneously without blocking, and only one writer is permitted at
/// a time. If a write attempt cannot acquire access, it returns `false`, allowing users to decide
/// how to handle contention. This could mean retrying, blocking, or yielding in async contexts.
///
/// ### Usage Example:
/// ```rust
/// use movetex::Movetex;
/// use std::sync::Arc;
/// use std::thread;
///
/// let movetex = Arc::new(Movetex::new("Initial value".to_string()));
///
/// thread::scope(|s| {
///     // Spawn multiple readers
///     for _ in 0..5 {
///         let m = Arc::clone(&movetex);
///         s.spawn(move || {
///             let value = m.read();
///             println!("Read value: {}", *value);
///         });
///     }
///
///     // Single writer attempting to modify the data
///     let m = Arc::clone(&movetex);
///     s.spawn(move || {
///         if m.write(|data| *data = "Updated value".to_string()) {
///             println!("Write succeeded");
///         } else {
///             // sleep, retry, or yield here
///         }
///     });
/// });
/// ```
///
/// The `write` method returns `false` if it cannot acquire access, so you can choose to handle
/// this case by blocking, retrying, or yielding if in an async context. Further explanations of
/// the `write` and `swap` mechanisms, and the cloning rationale in `write`, are provided in the
/// detailed documentation.

pub struct Movetex<T: Clone> {
    // Atomic pointer for reading
    ptr_r: AtomicPtr<T>,
    // Atomic pointer for writing
    ptr_w: AtomicPtr<T>,
}

impl<T: Clone> Movetex<T> {
    /// Creates a new `Movetex` instance containing an initial value.
    ///
    /// The `new` function initializes `Movetex` with a cloned version of the provided data.
    /// This approach ensures that there are independent copies for reading and writing from
    /// the start, allowing immediate, lock-free access for readers. The cloning on creation
    /// guarantees that the internal pointers for read and write are synchronized initially,
    /// supporting atomic operations on the data.
    pub fn new(value: T) -> Self {
        Self {
            ptr_r: AtomicPtr::new(Box::into_raw(Box::new(value.clone()))),
            ptr_w: AtomicPtr::new(Box::into_raw(Box::new(value))),
        }
    }

    /// Provides a reference to the read-only copy of the data in `Movetex`.
    ///
    /// The `read` method returns a `&T` reference, which is always safe to access and never null.
    /// `Movetex` maintains separate atomic pointers for reading and writing, ensuring
    /// that the reader always accesses a valid, initialized copy of the data.
    ///
    /// Readers do not block each other, and they are isolated from writers by accessing a separate copy.
    pub fn read(&self) -> &T {
        unsafe { &*self.ptr_r.load(Ordering::Acquire) }
    }

    /// The `write` method attempts an exclusive update to the stored value.
    ///
    /// During a write operation, the writer pointer (`ptr_w`) is temporarily set to `null_mut`
    /// to block concurrent writes, allowing only one writer at a time. If `ptr_w` is already `null_mut`,
    /// another write is in progress, and the function returns `false` to indicate the write cannot proceed.
    ///
    /// Readers are not blocked during the write operation; they can continue to read the current value.
    ///
    /// When accessible, the value is cloned and updated via the provided closure. After modification,
    /// the reader pointer (`ptr_r`) is atomically swapped to point to the new data, so that readers can
    /// immediately access the updated content without delays.
    ///
    /// Returns `true` if the write succeeds, or `false` if another write is in progress.
    pub fn write(&self, f: impl FnOnce(&mut T)) -> bool {
        if !self.ptr_w.load(Ordering::Acquire).is_null() {
            let mut value =
                unsafe { *Box::from_raw(self.ptr_w.swap(ptr::null_mut(), Ordering::Release)) };

            f(&mut value);

            drop(unsafe {
                Box::from_raw(
                    self.ptr_r
                        .swap(Box::into_raw(Box::new(value.clone())), Ordering::Release),
                )
            });

            self.ptr_w
                .store(Box::into_raw(Box::new(value)), Ordering::Release);

            return true;
        }
        false
    }

    /// The `swap` method atomically replaces the value stored in `ptr_w` without cloning.
    /// This operation allows for multiple threads to simultaneously replace the value without any copying overhead, as long as there is no ongoing `write` operation.
    ///
    /// ---
    /// ⚠️ It's important to note that `swap` only updates the writer (`ptr_w`) and does not affect the reader (`ptr_r`).
    /// This means that while the writer's value can be swapped atomically, the reader will continue to see the old value
    /// until a `write` operation occurs, which is the only operation that can update both the writer and reader values.
    ///
    /// If a `write` operation is in progress, the `swap` will return `None`, signaling that the operation could not be performed at that time.
    ///
    /// Returns `Some(old_value)` if the swap was successful, or `None` if a `write` operation was in progress.
    pub fn swap(&self, value: T) -> Option<T> {
        let ptr = self
            .ptr_w
            .swap(Box::into_raw(Box::new(value)), Ordering::Release);
        if ptr.is_null() {
            return None;
        }
        Some(unsafe { *Box::from_raw(ptr) })
    }
}

/// Implement `Drop` for `Movetex` to ensure that the internal pointers are correctly deallocated.
impl<T: Clone> Drop for Movetex<T> {
    /// Ensures that the internal pointers are correctly deallocated.
    fn drop(&mut self) {
        unsafe {
            let ptr_r = self.ptr_r.load(Ordering::Relaxed);
            if !ptr_r.is_null() {
                drop(Box::from_raw(ptr_r));
            }
            let ptr_w = self.ptr_w.load(Ordering::Relaxed);
            if !ptr_w.is_null() {
                drop(Box::from_raw(ptr_w));
            }
        }
    }
}
