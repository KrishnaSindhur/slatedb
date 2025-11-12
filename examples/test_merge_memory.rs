use bytes::Bytes;
use slatedb::{config::DbOptions, db::Db, MergeOperator, MergeOperatorError};
use std::sync::Arc;
use std::time::Instant;

/// Simple counter merge operator for testing
struct CounterMergeOperator;

impl MergeOperator for CounterMergeOperator {
    fn merge(
        &self,
        existing_value: Option<Bytes>,
        value: Bytes,
    ) -> Result<Bytes, MergeOperatorError> {
        let existing = existing_value
            .map(|v| u64::from_le_bytes(v.as_ref().try_into().unwrap()))
            .unwrap_or(0);
        let increment = u64::from_le_bytes(value.as_ref().try_into().unwrap());
        let result = existing + increment;
        Ok(Bytes::copy_from_slice(&result.to_le_bytes()))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Testing Merge Operator Memory Behavior ===\n");

    // Create a temporary directory for the database
    let temp_dir = std::env::temp_dir().join("slatedb_merge_test");
    if temp_dir.exists() {
        std::fs::remove_dir_all(&temp_dir)?;
    }
    std::fs::create_dir_all(&temp_dir)?;

    let path = format!("file://{}", temp_dir.display());
    println!("Database path: {}\n", path);

    // Open database with merge operator
    let mut options = DbOptions::default();
    options.merge_operator = Some(Arc::new(CounterMergeOperator));
    
    let db = Db::open_with_opts(path, options).await?;

    // Test 1: Write many merge operands for a single key
    println!("Test 1: Writing 10,000 merge operands to a single key");
    println!("-------------------------------------------------------");
    
    let key = b"counter_key";
    let num_operations = 10_000;
    
    let start = Instant::now();
    for i in 0..num_operations {
        let value = Bytes::copy_from_slice(&1u64.to_le_bytes());
        db.merge(key, value).await?;
        
        if (i + 1) % 1000 == 0 {
            println!("  Written {} merge operations...", i + 1);
        }
    }
    let write_duration = start.elapsed();
    println!("  ✓ Completed in {:?}\n", write_duration);

    // Test 2: Read the key (this triggers merge)
    println!("Test 2: Reading the key (triggers merge of all operands)");
    println!("----------------------------------------------------------");
    println!("  ⚠️  OLD CODE: This will collect ALL 10,000 entries in memory!");
    println!("  ⚠️  Memory usage: ~10,000 RowEntry objects at once");
    println!("  ⚠️  Then reverses the entire vector");
    println!("  ⚠️  Then merges pairwise (10,000 function calls)\n");
    
    let start = Instant::now();
    let result = db.get(key).await?;
    let read_duration = start.elapsed();
    
    if let Some(value) = result {
        let counter = u64::from_le_bytes(value.as_ref().try_into().unwrap());
        println!("  ✓ Counter value: {}", counter);
        println!("  ✓ Expected: {}", num_operations);
        println!("  ✓ Read completed in {:?}\n", read_duration);
        
        if counter == num_operations {
            println!("  ✅ Merge result is correct!");
        } else {
            println!("  ❌ Merge result is incorrect!");
        }
    }

    // Test 3: Multiple reads to see consistency
    println!("\nTest 3: Multiple reads (each triggers full merge)");
    println!("---------------------------------------------------");
    for i in 1..=3 {
        let start = Instant::now();
        let _ = db.get(key).await?;
        let duration = start.elapsed();
        println!("  Read #{}: {:?}", i, duration);
    }

    println!("\n=== Key Observations (OLD CODE) ===");
    println!("1. Memory: O(N) - collects all 10,000 entries");
    println!("2. Computation: 10,000 pairwise merge() calls");
    println!("3. Allocation: Large Vec allocation + reverse operation");
    println!("4. No batching or optimization possible");
    
    println!("\n=== What NEW CODE Would Improve ===");
    println!("1. Adds merge_batch() API for optimized implementations");
    println!("2. Processes in chunks of 100 during merge phase");
    println!("3. Reduces function calls from 10,000 to ~100");
    println!("4. Allows O(1) batch operations (e.g., sum all at once)");
    println!("5. Still collects all entries (needs reverse iterator for full fix)");

    // Cleanup
    db.close().await?;
    std::fs::remove_dir_all(&temp_dir)?;

    Ok(())
}
