# TinyDB Implementation Guide

Complete code examples for building TinyDB from scratch.

## Setup

```bash
cd /Users/krishna/Documents/opensource/slatedb
mkdir tinydb
cd tinydb
cargo init --lib
```

**Cargo.toml:**
```toml
[package]
name = "tinydb"
version = "0.1.0"
edition = "2021"

[dependencies]
bytes = "1.5"

[dev-dependencies]
tempfile = "3"
```

---

## Step 1: MemTable

**File:** `src/mem_table.rs`

```rust
use std::collections::BTreeMap;
use bytes::Bytes;

#[derive(Debug, Clone)]
pub struct Entry {
    pub value: Option<Bytes>,  // None = tombstone
    pub seq: u64,
}

pub struct MemTable {
    data: BTreeMap<Bytes, Entry>,
    size: usize,
}

impl MemTable {
    pub fn new() -> Self {
        Self {
            data: BTreeMap::new(),
            size: 0,
        }
    }

    pub fn put(&mut self, key: Bytes, value: Bytes, seq: u64) {
        self.size += key.len() + value.len() + 8;
        self.data.insert(key, Entry {
            value: Some(value),
            seq,
        });
    }

    pub fn delete(&mut self, key: Bytes, seq: u64) {
        self.size += key.len() + 8;
        self.data.insert(key, Entry {
            value: None,
            seq,
        });
    }

    pub fn get(&self, key: &[u8]) -> Option<&Entry> {
        self.data.get(key)
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Bytes, &Entry)> {
        self.data.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_put_get() {
        let mut mt = MemTable::new();
        mt.put(Bytes::from("key1"), Bytes::from("value1"), 1);
        
        let entry = mt.get(b"key1").unwrap();
        assert_eq!(entry.value.as_ref().unwrap(), &Bytes::from("value1"));
    }
}
```

---

## Step 2: Write-Ahead Log

**File:** `src/wal.rs`

```rust
use std::fs::{File, OpenOptions};
use std::io::{self, BufWriter, Write, BufReader, Read};
use std::path::Path;
use bytes::{Bytes, BytesMut, BufMut};

pub struct WAL {
    writer: BufWriter<File>,
}

impl WAL {
    pub fn create(path: impl AsRef<Path>) -> io::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        Ok(Self {
            writer: BufWriter::new(file),
        })
    }

    pub fn append_put(&mut self, key: &[u8], value: &[u8], seq: u64) -> io::Result<()> {
        let mut buf = BytesMut::new();
        buf.put_u8(1);  // PUT
        buf.put_u64(seq);
        buf.put_u32(key.len() as u32);
        buf.put_slice(key);
        buf.put_u32(value.len() as u32);
        buf.put_slice(value);
        
        self.writer.write_all(&buf)?;
        self.writer.flush()
    }

    pub fn append_delete(&mut self, key: &[u8], seq: u64) -> io::Result<()> {
        let mut buf = BytesMut::new();
        buf.put_u8(2);  // DELETE
        buf.put_u64(seq);
        buf.put_u32(key.len() as u32);
        buf.put_slice(key);
        
        self.writer.write_all(&buf)?;
        self.writer.flush()
    }
}
```

---

## Step 3: SSTable

**File:** `src/sst.rs`

```rust
use std::fs::File;
use std::io::{self, Write, BufWriter, BufReader, Read};
use std::path::Path;
use bytes::{Bytes, BytesMut, BufMut};

pub struct SSTWriter {
    writer: BufWriter<File>,
    entries_written: usize,
}

impl SSTWriter {
    pub fn create(path: impl AsRef<Path>) -> io::Result<Self> {
        let file = File::create(path)?;
        Ok(Self {
            writer: BufWriter::new(file),
            entries_written: 0,
        })
    }

    pub fn write_entry(&mut self, key: &[u8], value: Option<&[u8]>, seq: u64) -> io::Result<()> {
        let mut buf = BytesMut::new();
        buf.put_u64(seq);
        buf.put_u32(key.len() as u32);
        buf.put_slice(key);
        
        if let Some(v) = value {
            buf.put_u32(v.len() as u32);
            buf.put_slice(v);
        } else {
            buf.put_u32(0);  // Tombstone
        }
        
        self.writer.write_all(&buf)?;
        self.entries_written += 1;
        Ok(())
    }

    pub fn finish(mut self) -> io::Result<usize> {
        self.writer.flush()?;
        Ok(self.entries_written)
    }
}

pub struct SSTReader {
    reader: BufReader<File>,
}

impl SSTReader {
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let file = File::open(path)?;
        Ok(Self {
            reader: BufReader::new(file),
        })
    }

    pub fn scan(&mut self) -> io::Result<Vec<(Bytes, Option<Bytes>, u64)>> {
        let mut entries = Vec::new();

        loop {
            let mut seq_buf = [0u8; 8];
            if self.reader.read_exact(&mut seq_buf).is_err() {
                break;
            }
            let seq = u64::from_be_bytes(seq_buf);

            let mut key_len_buf = [0u8; 4];
            self.reader.read_exact(&mut key_len_buf)?;
            let key_len = u32::from_be_bytes(key_len_buf) as usize;
            let mut key = vec![0u8; key_len];
            self.reader.read_exact(&mut key)?;

            let mut value_len_buf = [0u8; 4];
            self.reader.read_exact(&mut value_len_buf)?;
            let value_len = u32::from_be_bytes(value_len_buf) as usize;

            let value = if value_len > 0 {
                let mut value = vec![0u8; value_len];
                self.reader.read_exact(&mut value)?;
                Some(Bytes::from(value))
            } else {
                None
            };

            entries.push((Bytes::from(key), value, seq));
        }

        Ok(entries)
    }
}
```

---

## Step 4: Main Database

**File:** `src/lib.rs`

```rust
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use bytes::Bytes;

mod mem_table;
mod wal;
mod sst;

use mem_table::MemTable;
use wal::WAL;
use sst::{SSTWriter, SSTReader};

const MEMTABLE_FLUSH_THRESHOLD: usize = 1024 * 1024;  // 1MB

pub struct TinyDB {
    mem_table: MemTable,
    wal: WAL,
    sst_dir: PathBuf,
    sst_files: Vec<PathBuf>,
    next_seq: AtomicU64,
    next_sst_id: AtomicU64,
}

impl TinyDB {
    pub fn open(dir: impl AsRef<Path>) -> std::io::Result<Self> {
        let dir = dir.as_ref();
        std::fs::create_dir_all(dir)?;

        let wal_path = dir.join("wal.log");
        let wal = WAL::create(&wal_path)?;

        Ok(Self {
            mem_table: MemTable::new(),
            wal,
            sst_dir: dir.to_path_buf(),
            sst_files: Vec::new(),
            next_seq: AtomicU64::new(1),
            next_sst_id: AtomicU64::new(1),
        })
    }

    pub fn put(&mut self, key: impl Into<Bytes>, value: impl Into<Bytes>) -> std::io::Result<()> {
        let key = key.into();
        let value = value.into();
        let seq = self.next_seq.fetch_add(1, Ordering::SeqCst);

        self.wal.append_put(&key, &value, seq)?;
        self.mem_table.put(key, value, seq);

        if self.mem_table.size() > MEMTABLE_FLUSH_THRESHOLD {
            self.flush()?;
        }

        Ok(())
    }

    pub fn delete(&mut self, key: impl Into<Bytes>) -> std::io::Result<()> {
        let key = key.into();
        let seq = self.next_seq.fetch_add(1, Ordering::SeqCst);

        self.wal.append_delete(&key, seq)?;
        self.mem_table.delete(key, seq);

        if self.mem_table.size() > MEMTABLE_FLUSH_THRESHOLD {
            self.flush()?;
        }

        Ok(())
    }

    pub fn get(&self, key: &[u8]) -> std::io::Result<Option<Bytes>> {
        // Check MemTable
        if let Some(entry) = self.mem_table.get(key) {
            return Ok(entry.value.clone());
        }

        // Check SST files (newest to oldest)
        for sst_path in self.sst_files.iter().rev() {
            let mut reader = SSTReader::open(sst_path)?;
            let entries = reader.scan()?;
            
            for (k, v, _seq) in entries {
                if k.as_ref() == key {
                    return Ok(v);
                }
            }
        }

        Ok(None)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if self.mem_table.size() == 0 {
            return Ok(());
        }

        let sst_id = self.next_sst_id.fetch_add(1, Ordering::SeqCst);
        let sst_path = self.sst_dir.join(format!("{:06}.sst", sst_id));

        let mut writer = SSTWriter::create(&sst_path)?;
        for (key, entry) in self.mem_table.iter() {
            writer.write_entry(
                key,
                entry.value.as_ref().map(|v| v.as_ref()),
                entry.seq,
            )?;
        }
        writer.finish()?;

        self.sst_files.push(sst_path);
        self.mem_table = MemTable::new();

        println!("Flushed to SST {}", sst_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_put_get() {
        let temp_dir = TempDir::new().unwrap();
        let mut db = TinyDB::open(temp_dir.path()).unwrap();

        db.put("key1", "value1").unwrap();
        assert_eq!(db.get(b"key1").unwrap(), Some(Bytes::from("value1")));
    }

    #[test]
    fn test_delete() {
        let temp_dir = TempDir::new().unwrap();
        let mut db = TinyDB::open(temp_dir.path()).unwrap();

        db.put("key1", "value1").unwrap();
        db.delete("key1").unwrap();
        assert_eq!(db.get(b"key1").unwrap(), None);
    }
}
```

---

## Running Tests

```bash
cargo test
```

## Next Steps

1. Add iterators
2. Implement merge operators
3. Add compaction
4. Compare with SlateDB

See `LEARNING_GUIDE.md` for the complete learning path!
