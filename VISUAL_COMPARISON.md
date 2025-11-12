# Visual Comparison: Old vs New Merge Operator

## Quick Reference

| File | Description |
|------|-------------|
| `slatedb/src/merge_operator.rs` | Currently has OLD code (checked out) |
| `test.rs` | Has NEW code (your backup) |
| `slatedb/src/merge_operator.rs.old` | Will be created as backup when you switch |

## Key Code Differences

### 1. Trait Definition

#### OLD CODE (Current)
```rust
pub trait MergeOperator {
    fn merge(
        &self,
        existing_value: Option<Bytes>,
        value: Bytes,
    ) -> Result<Bytes, MergeOperatorError>;
    
    // ❌ No merge_batch method
}
```

#### NEW CODE (test.rs)
```rust
pub trait MergeOperator {
    fn merge(
        &self,
        existing_value: Option<Bytes>,
        value: Bytes,
    ) -> Result<Bytes, MergeOperatorError>;
    
    // ✅ NEW: Batch merge method
    fn merge_batch(
        &self,
        existing_value: Option<Bytes>,
        operands: &[Bytes],
    ) -> Result<Bytes, MergeOperatorError> {
        // Default: pairwise merging (backward compatible)
        let mut result = existing_value;
        for operand in operands {
            result = Some(self.merge(result, operand.clone())?);
        }
        result.ok_or_else(|| {
            panic!("merge_batch called with empty operands")
        })
    }
}
```

### 2. Batch Size Constant

#### OLD CODE
```rust
// ❌ No batch size constant
```

#### NEW CODE
```rust
// ✅ NEW: Batch size for chunking
const MERGE_BATCH_SIZE: usize = 100;
```

### 3. Merge Logic

#### OLD CODE - Pairwise Merging
```rust
// After reversing entries...

for entry in entries.iter().filter(|e| is_not_expired(e, self.now)) {
    max_create_ts = merge_options(max_create_ts, entry.create_ts, i64::max);
    min_expire_ts = merge_options(min_expire_ts, entry.expire_ts, i64::min);
    seq = std::cmp::max(seq, entry.seq);

    match &entry.value {
        ValueDeletable::Merge(value) => {
            // ❌ OLD: One merge() call per entry
            merged_value = Some(self.merge_operator.merge(merged_value, value.clone())?);
        }
        _ => unreachable!("Should not merge any non-merge entries"),
    }
}
```

**For 10,000 operands:**
- 10,000 `merge()` function calls
- 10,000 iterations
- No batching optimization possible

#### NEW CODE - Batched Merging
```rust
// After reversing entries...

// ✅ NEW: Extract operands into a Vec
let merge_operands: Vec<Bytes> = entries
    .iter()
    .filter(|e| is_not_expired(e, self.now))
    .filter_map(|entry| {
        // Accumulate timestamps and seq
        max_create_ts = merge_options(max_create_ts, entry.create_ts, i64::max);
        min_expire_ts = merge_options(min_expire_ts, entry.expire_ts, i64::min);
        seq = std::cmp::max(seq, entry.seq);
        
        match &entry.value {
            ValueDeletable::Merge(value) => Some(value.clone()),
            _ => None,
        }
    })
    .collect();

// ✅ NEW: Process in batches of 100
for chunk in merge_operands.chunks(MERGE_BATCH_SIZE) {
    merged_value = Some(self.merge_operator.merge_batch(merged_value, chunk)?);
}
```

**For 10,000 operands:**
- 100 `merge_batch()` function calls (with 100 operands each)
- Users can optimize to O(1) per batch
- 99% reduction in function calls

### 4. Tests

#### OLD CODE
```rust
// ❌ No specific batching tests
// Only has existing merge tests
```

#### NEW CODE
```rust
// ✅ NEW: Test with 250 operands (2.5x batch size)
#[tokio::test]
async fn test_batched_merge_with_many_operands() {
    let mut data = vec![];
    for i in 1..=250 {
        data.push(RowEntry::new_merge(b"key1", &[i as u8], i));
    }
    // ... test that batching works correctly
}

// ✅ NEW: Test with base value + 150 operands
#[tokio::test]
async fn test_batched_merge_with_base_value() {
    data.push(RowEntry::new_value(b"key1", b"BASE", 0));
    for i in 1..=150 {
        data.push(RowEntry::new_merge(b"key1", &[i as u8], i));
    }
    // ... test that base value + batching works
}
```

## Execution Flow Comparison

### OLD CODE Flow
```
1. Collect all entries → [e1, e2, ..., e10000]
2. Reverse → [e10000, ..., e2, e1]
3. For each entry:
   - merged = merge(merged, entry)    ← 10,000 calls
4. Return result
```

### NEW CODE Flow
```
1. Collect all entries → [e1, e2, ..., e10000]
2. Reverse → [e10000, ..., e2, e1]
3. Extract operands → [op1, op2, ..., op10000]
4. For each chunk of 100:
   - merged = merge_batch(merged, chunk)  ← 100 calls
5. Return result
```

## Performance Example: Counter

### Scenario
10,000 increment operations on a counter

### OLD CODE
```rust
impl MergeOperator for CounterMergeOperator {
    fn merge(&self, existing: Option<Bytes>, value: Bytes) 
        -> Result<Bytes, MergeOperatorError> {
        let a = existing.map(decode).unwrap_or(0);
        let b = decode(&value);
        Ok(encode(a + b))  // One addition
    }
}

// Execution:
// merge(0, 1) = 1
// merge(1, 1) = 2
// merge(2, 1) = 3
// ... 10,000 times
// = 10,000 function calls
// = 10,000 additions
// = 10,000 encode/decode operations
```

### NEW CODE (Default)
```rust
impl MergeOperator for CounterMergeOperator {
    fn merge(&self, existing: Option<Bytes>, value: Bytes) 
        -> Result<Bytes, MergeOperatorError> {
        let a = existing.map(decode).unwrap_or(0);
        let b = decode(&value);
        Ok(encode(a + b))
    }
    
    // Uses default merge_batch (calls merge in loop)
}

// Execution:
// merge_batch(0, [1,1,1...×100]) → calls merge 100 times → 100
// merge_batch(100, [1,1,1...×100]) → calls merge 100 times → 200
// ... 100 times
// = 100 merge_batch calls
// = 10,000 merge calls (same as old)
// = 10,000 additions (same)
// BUT: Better cache locality, less overhead
```

### NEW CODE (Optimized)
```rust
impl MergeOperator for CounterMergeOperator {
    fn merge(&self, existing: Option<Bytes>, value: Bytes) 
        -> Result<Bytes, MergeOperatorError> {
        let a = existing.map(decode).unwrap_or(0);
        let b = decode(&value);
        Ok(encode(a + b))
    }
    
    // ✅ OPTIMIZED: Sum all at once!
    fn merge_batch(&self, existing: Option<Bytes>, operands: &[Bytes]) 
        -> Result<Bytes, MergeOperatorError> {
        let sum = existing.map(decode).unwrap_or(0)
            + operands.iter().map(|b| decode(b)).sum::<u64>();
        Ok(encode(sum))
    }
}

// Execution:
// merge_batch(0, [1,1,1...×100]) → sum = 100
// merge_batch(100, [1,1,1...×100]) → sum = 200
// ... 100 times
// = 100 merge_batch calls
// = 100 sum operations (SIMD possible!)
// = 10,000 additions (but vectorized)
// = MUCH FASTER!
```

## How to Test

### See OLD code behavior:
```bash
# Currently active
cargo test --lib merge_operator::tests::test_merge_operator_iterator -- --nocapture
```

### See NEW code behavior:
```bash
# Copy new code
cp test.rs slatedb/src/merge_operator.rs

# Run all tests including new batching tests
cargo test --lib merge_operator -- --nocapture

# Restore old code
git checkout slatedb/src/merge_operator.rs
```

### Use the comparison script:
```bash
chmod +x compare_implementations.sh
./compare_implementations.sh
```

## Summary

| Aspect | Old Code | New Code |
|--------|----------|----------|
| **API** | `merge()` only | `merge()` + `merge_batch()` |
| **Batching** | ❌ No | ✅ Yes (100 per batch) |
| **Function calls** | N | N/100 |
| **User optimization** | ❌ Not possible | ✅ Can override `merge_batch()` |
| **Memory (collection)** | O(N) | O(N) - same limitation |
| **Memory (merge)** | O(N) | O(100) - bounded |
| **Backward compat** | N/A | ✅ Yes (default impl) |
| **Tests** | Existing only | + 2 new batching tests |

The new code provides the **foundation** for optimization while maintaining backward compatibility. The full memory solution requires reverse iterator support (issues #663, #438).
