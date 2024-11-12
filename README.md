# Movetex

Movetex provides a non-blocking, atomic-based synchronization primitive in Rust, enabling multiple readers or a single writer to access complex data structures safely and efficiently. Unlike a standard `Mutex`, Movetex performs atomic pointer swaps, making it suitable for advanced scenarios where atomic operations are needed not only for basic types but also for complex data structures.

## Key Features

- **Atomic Operations for Complex Data**: Movetex uses two `AtomicPtr` pointers, allowing atomic read/write access for non-trivial data types without blocking readers.
- **Multiple Readers, Single Writer**: Allows multiple readers simultaneously, while only one writer can have access at a time, without reader blocking.
- **Conditional Write Handling**: `write` returns a `bool` to indicate if the write lock was acquired, giving you control over what to do if a write isn’t immediately possible.
   - For multithreaded scenarios, you can retry or pause.
   - In async contexts, yield the current green thread to improve efficiency.

## Example

```rust
use movetex::Movetex;
use std::sync::Arc;
use std::thread;

let movetex = Arc::new(Movetex::new("Initial value".to_string()));

thread::scope(|s| {
    // Spawn multiple readers
    for _ in 0..5 {
        let m = Arc::clone(&movetex);
        s.spawn(move || {
            let value = m.read();
            println!("Read value: {}", *value);
        });
    }

    // Single writer attempting to modify the data
    let m = Arc::clone(&movetex);
    s.spawn(move || {
        if m.write(|data| *data = "Updated value".to_string()) {
            println!("Write succeeded");
        } else {
            // sleep, retry, or yield here
        }
    });
});
```

# License
[MIT License](https://github.com/m62624/movetex/blob/main/LICENSE)