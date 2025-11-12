# Test Fix Explanation

## The Bug in the Test

The `test_batched_merge_with_base_value` test had incorrect sequence numbers.

### What Was Wrong

```rust
// WRONG: Base value had HIGHEST seq (151)
for i in 1..=150 {
    data.push(RowEntry::new_merge(b"key1", &[i as u8], i));  // seq: 1-150
}
data.push(RowEntry::new_value(b"key1", b"BASE", 151));       // seq: 151 (HIGHEST!)
```

After sorting by descending seq:
```
[value(151), merge(150), merge(149), ..., merge(1)]
         ↑ Base value comes FIRST (wrong!)
```

The iterator returns the base value FIRST, so it gets treated as the newest entry, and the merge operands are ignored because they come after a base value.

### The Fix

```rust
// CORRECT: Base value has LOWEST seq (0)
data.push(RowEntry::new_value(b"key1", b"BASE", 0));         // seq: 0 (LOWEST!)
for i in 1..=150 {
    data.push(RowEntry::new_merge(b"key1", &[i as u8], i));  // seq: 1-150
}
```

After sorting by descending seq:
```
[merge(150), merge(149), ..., merge(1), value(0)]
                                             ↑ Base value comes LAST (correct!)
```

After reversing (oldest→newest):
```
[value(0), merge(1), merge(2), ..., merge(150)]
     ↑ Base value is oldest
```

Merge result: `"BASE" + 1 + 2 + ... + 150` ✓

## Why This Matters

In a real database:
- **Base value** = oldest Put/Write operation (lowest seq)
- **Merge operands** = newer Merge operations (higher seq)

The test now correctly models this relationship.

## Expected Test Result

After the fix, both tests should pass:
- ✅ `test_batched_merge_with_many_operands` - 250 merge operands
- ✅ `test_batched_merge_with_base_value` - base value + 150 merge operands
