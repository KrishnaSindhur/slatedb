# Learning SlateDB: Build Your Own LSM-Tree Database

## 🎯 Goal
Understand SlateDB by reimplementing a simplified version called **TinyDB** from scratch.

## 📋 Prerequisites
- Basic Rust knowledge (ownership, traits, async/await)
- Understanding of key-value stores
- Familiarity with Git

## 🗺️ Learning Path Overview

```
Phase 1: Core Concepts (Week 1-2)
    ↓
Phase 2: Build TinyDB Basics (Week 3-4)
    ↓
Phase 3: Advanced Features (Week 5-6)
    ↓
Phase 4: Compare with SlateDB (Week 7-8)
```

---

## Phase 1: Core Concepts (Week 1-2)

### What is an LSM-Tree?
- **Log-Structured Merge Tree** - optimized for write-heavy workloads
- Writes go to memory first (fast), then flush to disk (durable)
- Reads check memory first, then disk files (newest to oldest)

### Key Components
1. **MemTable** - In-memory sorted map (BTreeMap)
2. **WAL (Write-Ahead Log)** - Durability (recover after crash)
3. **SSTable** - Sorted String Table (immutable disk files)
4. **Compaction** - Merge old files to reclaim space

### Study SlateDB Files (in order)
1. `types.rs` - Data structures (RowEntry, ValueDeletable)
2. `row_codec.rs` - Encoding/decoding
3. `mem_table.rs` - In-memory storage
4. `block.rs` - SSTable block format
5. `sst.rs` - Sorted String Table
6. `iter.rs` - Iterator trait
7. `merge_operator.rs` - Merge operators (you know this!)
8. `db.rs` - Main database API

---

## Phase 2: Build TinyDB (Week 3-4)

### Setup
```bash
cd /Users/krishna/Documents/opensource/slatedb
mkdir tinydb
cd tinydb
cargo init --lib

# Add dependencies
cat >> Cargo.toml << EOF
[dependencies]
bytes = "1.5"

[dev-dependencies]
tempfile = "3"
EOF
```

### Project Structure
```
tinydb/
├── Cargo.toml
├── src/
│   ├── lib.rs           # Main DB API
│   ├── mem_table.rs     # In-memory storage
│   ├── wal.rs           # Write-ahead log
│   └── sst.rs           # SSTable reader/writer
└── tests/
    └── integration_test.rs
```

See `TINYDB_IMPLEMENTATION.md` for complete code examples.

---

## Phase 3: Advanced Features (Week 5-6)

### Week 5: Iterators
- Study `iter.rs` and `merge_iterator.rs` in SlateDB
- Implement iterator for TinyDB
- Merge MemTable + SST iterators

### Week 6: Merge Operators
- Implement `MergeOperator` trait
- Add counter merge operator
- Add batch merge support (what you learned!)

---

## Phase 4: Compare with SlateDB (Week 7-8)

### Key Differences

| Feature | TinyDB | SlateDB |
|---------|--------|---------|
| Storage | Local files | Object storage (S3) |
| MemTable | Simple BTreeMap | Optimized with tracking |
| SST Format | Simple binary | Block-based with index |
| Compaction | None | Size-tiered |
| Concurrency | Single-threaded | Async with tokio |

### Study Advanced Topics
1. Compaction (`compactor.rs`)
2. Object storage (`cached_object_store/`)
3. Async patterns (throughout)
4. Testing strategies (`tests/`)

---

## 📚 Resources

### Papers
- [LSM-Tree Paper](https://www.cs.umb.edu/~poneil/lsmtree.pdf)
- [Bigtable Paper](https://research.google/pubs/pub27898/)

### Books
- Database Internals by Alex Petrov
- Designing Data-Intensive Applications by Martin Kleppmann

### Code
- [Mini-LSM](https://github.com/skyzh/mini-lsm)
- [RocksDB](https://github.com/facebook/rocksdb)
- [LevelDB](https://github.com/google/leveldb)

---

## 🎯 Milestones

- [ ] Week 2: Understand LSM architecture
- [ ] Week 4: TinyDB basic operations work
- [ ] Week 6: Iterators and merge operators
- [ ] Week 8: Understand SlateDB optimizations

---

## 💡 Tips

1. Start small - implement one component at a time
2. Write tests first
3. Read SlateDB tests to understand usage
4. Ask questions on Discord/GitHub
5. Document your learning

---

## 🚀 Next Steps

1. Read this guide completely
2. Study SlateDB's `types.rs` and `mem_table.rs`
3. Start implementing TinyDB MemTable
4. See `TINYDB_IMPLEMENTATION.md` for code examples
5. Track progress in `PROGRESS.md`

Good luck! 🎉
