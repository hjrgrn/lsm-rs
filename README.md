# Log-Structured Merge Tree

A patchy, single-threaded implementation of the log-structured merge tree concept, written in Rust.

Crash recovery is provided by a write-ahead log.

The state of the application is tracked and restored using a manifest file that is updated atomically.

Persistent data is stored in immutable sorted string tables (SSTables).

Compaction is handled using a k-way merge algorithm.

Concepts like TOMBSTONES and memtables are implemented leveraging Rust's type system.

The functioning of the program can be verified by the integration test suite.

This project is an occasion to practice concepts learned while studying Kleppmann, M. (2017). *Designing Data-Intensive Applications: The Big Ideas Behind Reliable, Scalable, and Maintainable Systems*. O'Reilly Media. ISBN 978-1-449-37332-0.

It is an exploratory project and **should not be used in production**.
