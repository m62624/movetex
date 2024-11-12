# Changelog

## Version 1.0.0

### Features
- Introduced `Movetex`: a non-blocking, atomic-based synchronization primitive for safe and efficient access to complex data structures.
- Supports multiple readers and a single writer without blocking readers.
- Conditional write handling: `write` method returns a `bool` indicating if the write lock was acquired.
- Example usage in documentation demonstrating core features of the library.