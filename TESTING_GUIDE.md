# Testing Guide: Old vs New Merge Operator

## Current Situation

You have:
- ✅ **Old code**: Currently checked out (original implementation)
- ✅ **New code**: Saved in `test.rs` (batched merge implementation)

## Step-by-Step Testing Process

### Step 1: Understand the Old Code Behavior

The old code in `merge_with_older_entries` does:

```rust
// 1. Collect ALL entries into a Vec
let mut entries = vec![first_entry];
loop {
    // ... collect all merge operands ...
    entries.push(next_entry);
}

// 2. Reverse the entire Vec
entries.reverse();

// 3. Merge pairwise (N function calls)
for entry in entries.iter() {
    merged_value = Some(self.merge_operator.merge(merged_value, value.clone())?);
}
```

**Problems:**
- Memory: O(N) - all entries in memory
- Computation: N merge() calls
- No optimization possible

### Step 2: Add Debug Logging to See the Issue

Add this to `slatedb/src/merge_operator.rs` at line ~176 (in `merge_with_older_entries`):

```rust
// After collecting all entries
entries.reverse();

// ADD THIS DEBUG OUTPUT:
eprintln!("🔍 OLD CODE: Collected {} entries for key {:?}", 
          entries.len(), 
          String::from_utf8_lossy(&key));
eprintln!("   Memory usage: ~{} bytes for RowEntry objects", 
          entries.len() * std::mem::size_of::<RowEntry>());
```

And after the merge loop:

```rust
// After merging
if let Some(result_value) = merged_value {
    // ADD THIS:
    eprintln!("   Performed {} pairwise merge() calls", entries.len());
    
    return Ok(Some(RowEntry::new(...)));
}
```

### Step 3: Create a Simple Test

Create `test_old_merge.rs` in the root:

```rust
use slatedb::merge_operator::tests::*;

#[tokio::main]
async fn main() {
    // Test with many operands
    let merge_operator = Arc::new(MockMergeOperator {});
    
    // Create 1000 merge operands
    let mut data = vec![];
    for i in 1..=1000 {
        data.push(RowEntry::new_merge(b"key1", &[i as u8], i));
    }
    
    println!("Testing OLD merge with 1000 operands...");
    let mut iterator = MergeOperatorIterator::new(
        merge_operator,
        data.into(),
        true,
        0,
    );
    
    iterator.init().await.unwrap();
    let result = iterator.next_entry().await.unwrap();
    
    println!("Result: {:?}", result);
}
```

### Step 4: Run Tests with Old Code

```bash
# Run existing tests to see behavior
cargo test --lib merge_operator -- --nocapture

# Look for the debug output showing:
# - How many entries collected
# - Memory usage
# - Number of merge calls
```

### Step 5: Profile Memory Usage (Optional)

If you want to see actual memory usage:

```bash
# Install valgrind/heaptrack (Linux) or Instruments (Mac)

# On Mac:
cargo build --release --lib
instruments -t Allocations ./target/release/deps/slatedb-*

# On Linux:
cargo build --release --lib
valgrind --tool=massif cargo test --release merge_operator
```

### Step 6: Compare with New Code

After observing old behavior:

```bash
# Copy new code back
cp test.rs slatedb/src/merge_operator.rs

# Run same tests
cargo test --lib merge_operator -- --nocapture

# Compare:
# - Memory usage (should be similar - still collects all)
# - Number of merge calls (should be ~10x less with batching)
# - Performance (should be faster due to fewer function calls)
```

## Quick Comparison Commands

### Test Old Code (Current)
```bash
cd /Users/krishna/Documents/opensource/slatedb

# See what's currently in the file
head -100 slatedb/src/merge_operator.rs

# Run tests
cargo test --lib merge_operator::tests::test_merge_operator_iterator -- --nocapture

# Check line count (old code is shorter)
wc -l slatedb/src/merge_operator.rs
```

### Test New Code
```bash
# Replace with new code
cp test.rs slatedb/src/merge_operator.rs

# Run tests (includes new batching tests)
cargo test --lib merge_operator -- --nocapture

# Check line count (new code is longer)
wc -l slatedb/src/merge_operator.rs
```

## What to Observe

### In Old Code:
1. **Collection phase**: Watch how it collects ALL entries
2. **Reverse operation**: Entire vector gets reversed
3. **Merge loop**: One merge() call per entry
4. **No batching**: Can't optimize even if you wanted to

### In New Code:
1. **Collection phase**: Same (still collects all - limitation noted)
2. **Reverse operation**: Same (still reverses all)
3. **Merge loop**: Uses `merge_batch()` with chunks of 100
4. **Batching**: Allows optimized implementations

## Key Differences

| Aspect | Old Code | New Code |
|--------|----------|----------|
| **API** | Only `merge()` | `merge()` + `merge_batch()` |
| **Collection** | O(N) memory | O(N) memory (same) |
| **Merge calls** | N calls | N/100 calls (~100 for 10K ops) |
| **Optimization** | Not possible | User can optimize `merge_batch()` |
| **Memory during merge** | N entries | 100 entries per batch |

## Example: Counter Performance

### Old Code
```rust
// 10,000 merge operands
// = 10,000 merge() function calls
// = 10,000 additions
// = 10,000 allocations
```

### New Code (Default)
```rust
// 10,000 merge operands
// = 100 merge_batch() calls (100 operands each)
// = 10,000 additions (same)
// = 100 allocations (99% reduction)
```

### New Code (Optimized)
```rust
impl MergeOperator for CounterMergeOperator {
    fn merge_batch(&self, existing: Option<Bytes>, operands: &[Bytes]) 
        -> Result<Bytes, MergeOperatorError> {
        // O(1) operation!
        let sum = existing.map(decode).unwrap_or(0) 
            + operands.iter().map(decode).sum::<u64>();
        Ok(encode(sum))
    }
}

// 10,000 merge operands
// = 100 merge_batch() calls
// = 100 sum operations (SIMD possible!)
// = 100 allocations
```

## Recommended Testing Order

1. ✅ **Understand current code** (you're here)
2. 🔄 **Add debug logging** to see collection size
3. 🔄 **Run existing tests** with old code
4. 🔄 **Note the behavior** (entries collected, merge calls)
5. 🔄 **Switch to new code** (`cp test.rs slatedb/src/merge_operator.rs`)
6. 🔄 **Run same tests** with new code
7. 🔄 **Compare results** (should be identical but faster)
8. ✅ **Understand the improvement**

## Quick Test Script

Save this as `compare_merge.sh`:

```bash
#!/bin/bash

echo "=== Testing OLD merge operator ==="
git checkout slatedb/src/merge_operator.rs
cargo test --lib merge_operator::tests::test_merge_operator_iterator
echo ""

echo "=== Testing NEW merge operator ==="
cp test.rs slatedb/src/merge_operator.rs
cargo test --lib merge_operator
echo ""

echo "=== Restoring OLD code ==="
git checkout slatedb/src/merge_operator.rs
```

Run with:
```bash
chmod +x compare_merge.sh
./compare_merge.sh
```
