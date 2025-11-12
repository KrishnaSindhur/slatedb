# TinyDB Learning Progress

Track your progress as you learn SlateDB by building TinyDB.

---

## Week 1: Core Concepts

### Day 1-3: LSM-Tree Architecture
- [ ] Read LSM-Tree paper (sections 1-3)
- [ ] Understand write path (WAL → MemTable → SSTable)
- [ ] Understand read path (MemTable → SSTables)
- [ ] Draw architecture diagram

**Notes:**


### Day 4-7: Study SlateDB Files
- [ ] `types.rs` - RowEntry, ValueDeletable
- [ ] `row_codec.rs` - Encoding/decoding
- [ ] `mem_table.rs` - In-memory storage
- [ ] `block.rs` - Block format
- [ ] `sst.rs` - SSTable implementation

**Questions:**


---

## Week 2: Deep Dive

### Day 8-10: Operations
- [ ] Trace `put()` operation in SlateDB
- [ ] Trace `get()` operation in SlateDB
- [ ] Trace `flush()` operation in SlateDB
- [ ] Write pseudocode for each

**Insights:**


### Day 11-14: Iterators and Merge
- [ ] Study `iter.rs`
- [ ] Study `merge_iterator.rs`
- [ ] Study `merge_operator.rs`
- [ ] Understand streaming batch merge

**Key Learnings:**


---

## Week 3: Build TinyDB Basics

### Day 1-2: MemTable
- [ ] Implement MemTable struct
- [ ] Implement put/get/delete
- [ ] Write tests
- [ ] Tests passing

**Challenges:**


### Day 3-4: WAL
- [ ] Implement WAL writer
- [ ] Implement WAL reader
- [ ] Write tests
- [ ] Tests passing

**Challenges:**


### Day 5-7: SSTable
- [ ] Implement SST writer
- [ ] Implement SST reader
- [ ] Write tests
- [ ] Tests passing

**Challenges:**


---

## Week 4: Complete TinyDB

### Day 1-4: Database Implementation
- [ ] Implement TinyDB struct
- [ ] Implement put/get/delete
- [ ] Implement flush
- [ ] Integration tests passing

**Challenges:**


### Day 5-7: Testing and Debugging
- [ ] Test with large datasets
- [ ] Test flush behavior
- [ ] Fix bugs
- [ ] All tests passing

**Bugs Fixed:**


---

## Week 5: Iterators

### Day 1-3: Iterator Design
- [ ] Design iterator interface
- [ ] Implement MemTable iterator
- [ ] Implement SST iterator

**Design Decisions:**


### Day 4-7: Merge Iterator
- [ ] Implement merge iterator
- [ ] Handle tombstones
- [ ] Write tests
- [ ] Tests passing

**Challenges:**


---

## Week 6: Merge Operators

### Day 1-3: Trait Implementation
- [ ] Define MergeOperator trait
- [ ] Implement counter merge
- [ ] Implement concat merge
- [ ] Write tests

**Learnings:**


### Day 4-7: Batch Merge
- [ ] Implement merge_batch
- [ ] Add streaming optimization
- [ ] Write tests
- [ ] Tests passing

**Optimizations:**


---

## Week 7: Compare with SlateDB

### Day 1-3: Architecture Comparison
- [ ] Compare storage layer
- [ ] Compare MemTable implementation
- [ ] Compare SST format
- [ ] Document differences

**Key Differences:**


### Day 4-7: Optimization Study
- [ ] Study SlateDB's block format
- [ ] Study caching strategy
- [ ] Study async patterns
- [ ] Document optimizations

**Insights:**


---

## Week 8: Advanced Topics

### Day 1-4: Compaction
- [ ] Study `compactor.rs`
- [ ] Understand size-tiered strategy
- [ ] Understand garbage collection
- [ ] Write notes

**Notes:**


### Day 5-7: Object Storage
- [ ] Study `cached_object_store/`
- [ ] Understand S3 integration
- [ ] Understand caching
- [ ] Write notes

**Notes:**


---

## Questions and Answers

### Questions
1. Why use BTreeMap instead of HashMap for MemTable?
   - Answer: 

2. How does SlateDB handle concurrent writes?
   - Answer: 

3. Why object storage instead of local files?
   - Answer: 

4. How does compaction decide which files to merge?
   - Answer: 

### Add your own questions here:


---

## Key Insights

1. LSM-Trees optimize for writes by...

2. The streaming batch merge reduces memory by...

3. Object storage enables...

### Add your insights:


---

## Next Steps

- [ ] Contribute to SlateDB
- [ ] Write blog post about learning
- [ ] Implement additional features in TinyDB
- [ ] Share TinyDB on GitHub

---

## Resources Used

- [ ] LSM-Tree paper
- [ ] Database Internals book
- [ ] SlateDB source code
- [ ] Mini-LSM tutorial
- [ ] RocksDB documentation

### Helpful links:


---

## Completion Checklist

- [ ] Understand LSM-Tree architecture
- [ ] TinyDB compiles and runs
- [ ] All tests passing
- [ ] Iterators work
- [ ] Merge operators implemented
- [ ] Can explain SlateDB optimizations
- [ ] Ready to contribute!

---

**Started:** [Date]
**Completed:** [Date]
**Total Time:** [Hours]
