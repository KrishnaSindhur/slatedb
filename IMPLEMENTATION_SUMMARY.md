# Batched Merge Implementation - Summary

## What Was Implemented

Added `merge_batch` method to the `MergeOperator` trait and refactored `merge_with_older_entries` to use batched merging.

## Key Achievement: Computational Efficiency

The main benefit of this implementation is **computational efficiency** through the `merge_batch` API:

### Before
```rust
// Pairwise merging - 250 function calls
result = merge(None, op1)
result = merge(result, op2)
result = merge(result, op3)
// ... 247 more calls
```

### After
```rust
// Batched merging - 3 function calls
result = merge_batch(None, [op1..op100])
result = merge_batch(result, [op101..op200])
result = merge_batch(result, [op201..op250])
```

### Benefits
1. **Reduced function call overhead**: 250 calls → 3 calls
2. **Optimized implementations**: Users can implement O(1) batch operations
   - Counter: Sum all values at once
   - String concat: Pre-calculate size, single allocation
3. **SIMD opportunities**: Batch operations can use vectorization

## Current Limitation: Memory

**The implementation still collects all entries into memory** during the collection phase.

### Why?
The iterator provides entries in **newest→oldest** order, but we need to:
1. Find if there's a base value (at the end)
2. Merge from **oldest→newest** order

This requires collecting all entries first, then reversing.

### Memory Usage
- **Collection**: O(N) - all RowEntry objects
- **Merge Phase**: O(BATCH_SIZE) - processes 100 at a time

So for 10,000 operands:
- ✅ **Merge phase**: Uses only ~100 operands at a time
- ❌ **Collection phase**: Still needs all 10,000 RowEntry objects

## Future Improvement: True Streaming

To achieve O(BATCH_SIZE) memory during collection, we need:

### Reverse Iterator Support
```rust
// Stream from oldest to newest
let mut reverse_iter = iterator.reverse();
let base_value = reverse_iter.next(); // Get oldest first

// Now stream and batch
for chunk in reverse_iter.chunks(100) {
    result = merge_batch(result, chunk);
}
```

This would provide:
- ✅ O(BATCH_SIZE) memory during collection
- ✅ O(BATCH_SIZE) memory during merging
- ✅ No need to collect all entries

### Related Issues
- #663: Reverse iterator support
- #438: Iterator improvements

## What This PR Achieves

### 1. API Foundation
- Introduces `merge_batch` trait method
- Backward compatible default implementation
- Enables future optimizations

### 2. Computational Efficiency
- Reduces merge function calls by ~99%
- Enables optimized batch implementations
- Reduces intermediate allocations

### 3. Partial Memory Improvement
- Merge phase uses bounded memory
- Prepares codebase for true streaming (when reverse iterator available)

## Recommendation

This implementation should be merged as:
1. **Immediate value**: Computational efficiency gains
2. **API stability**: Establishes `merge_batch` interface
3. **Future-ready**: Easy to upgrade when reverse iterator lands

The memory pressure issue from #963 is **partially addressed** (merge phase) and **fully addressable** once reverse iterator support is added.

## Testing

All existing tests pass. New tests verify:
- ✅ Batched merging with 250 operands
- ✅ Batched merging with base value
- ✅ Correct order preservation
- ✅ Backward compatibility

Run tests:
```bash
cargo test --lib merge_operator
```

## Example: Optimized Counter

Users can now implement efficient counters:

```rust
impl MergeOperator for CounterMergeOperator {
    fn merge(&self, existing: Option<Bytes>, value: Bytes) 
        -> Result<Bytes, MergeOperatorError> {
        let a = existing.map(decode_u64).unwrap_or(0);
        let b = decode_u64(&value);
        Ok(encode_u64(a + b))
    }
    
    fn merge_batch(&self, existing: Option<Bytes>, operands: &[Bytes]) 
        -> Result<Bytes, MergeOperatorError> {
        // O(N) single pass instead of O(N) function calls
        let sum: u64 = existing.map(decode_u64).unwrap_or(0)
            + operands.iter().map(|b| decode_u64(b)).sum::<u64>();
        Ok(encode_u64(sum))
    }
}
```

Performance for 10,000 increments:
- Before: 10,000 function calls, 10,000 allocations
- After: 100 function calls, 100 allocations (99% reduction)
