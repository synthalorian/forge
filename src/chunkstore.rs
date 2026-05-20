use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};

pub const DEFAULT_CHUNK_SIZE: usize = 4 * 1024 * 1024; // 4MB

pub struct ChunkStore {
    chunks_dir: PathBuf,
    compression_level: i32,
}

pub struct ChunkInfo {
    pub hash: String,
    pub original_size: usize,
    pub compressed_size: usize,
    pub is_new: bool,
}

impl ChunkStore {
    pub fn new(chunks_dir: PathBuf, compression_level: i32) -> Result<Self> {
        fs::create_dir_all(&chunks_dir)
            .with_context(|| format!("Failed to create chunks dir {}", chunks_dir.display()))?;
        Ok(Self {
            chunks_dir,
            compression_level,
        })
    }

    pub fn store_chunk(&self, data: &[u8]) -> Result<ChunkInfo> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash = hex::encode(hasher.finalize());

        if self.has_chunk(&hash) {
            return Ok(ChunkInfo {
                hash,
                original_size: data.len(),
                compressed_size: 0,
                is_new: false,
            });
        }

        let compressed =
            zstd::encode_all(data, self.compression_level).context("Failed to compress chunk")?;

        let path = self.hash_to_path(&hash);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create chunk subdir {}", parent.display()))?;
        }
        fs::write(&path, &compressed).with_context(|| format!("Failed to write chunk {}", hash))?;

        Ok(ChunkInfo {
            hash,
            original_size: data.len(),
            compressed_size: compressed.len(),
            is_new: true,
        })
    }

    pub fn read_chunk(&self, hash: &str) -> Result<Vec<u8>> {
        let path = self.hash_to_path(hash);
        let compressed =
            fs::read(&path).with_context(|| format!("Failed to read chunk file {}", hash))?;
        let data = zstd::decode_all(&compressed[..])
            .with_context(|| format!("Failed to decompress chunk {}", hash))?;
        Ok(data)
    }

    pub fn has_chunk(&self, hash: &str) -> bool {
        self.hash_to_path(hash).exists()
    }

    fn hash_to_path(&self, hash: &str) -> PathBuf {
        self.chunks_dir
            .join(&hash[0..2])
            .join(format!("{}.zst", &hash[2..]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use tempfile::TempDir;

    #[test]
    fn store_and_read_chunk() -> Result<()> {
        let tmp = TempDir::new()?;
        let store = ChunkStore::new(tmp.path().join("chunks"), 3)?;

        let data = b"Hello, ChunkStore! This is test data for verification.";
        let info = store.store_chunk(data)?;

        assert!(info.is_new);
        assert_eq!(info.original_size, data.len());
        assert!(info.compressed_size > 0);
        assert_eq!(info.hash.len(), 64);

        let read_data = store.read_chunk(&info.hash)?;
        assert_eq!(read_data, data.to_vec());

        Ok(())
    }

    #[test]
    fn dedup_returns_not_new() -> Result<()> {
        let tmp = TempDir::new()?;
        let store = ChunkStore::new(tmp.path().join("chunks"), 3)?;

        let data = b"Dedup test data";
        let first = store.store_chunk(data)?;
        assert!(first.is_new);

        let second = store.store_chunk(data)?;
        assert!(!second.is_new);
        assert_eq!(first.hash, second.hash);

        Ok(())
    }

    #[test]
    fn has_chunk_returns_correctly() -> Result<()> {
        let tmp = TempDir::new()?;
        let store = ChunkStore::new(tmp.path().join("chunks"), 3)?;

        assert!(!store.has_chunk("nonexistent"));

        let info = store.store_chunk(b"test data")?;
        assert!(store.has_chunk(&info.hash));

        Ok(())
    }

    #[test]
    fn compression_reduces_size() -> Result<()> {
        let tmp = TempDir::new()?;
        let store = ChunkStore::new(tmp.path().join("chunks"), 19)?;

        let data = "A".repeat(10000);
        let info = store.store_chunk(data.as_bytes())?;

        assert!(
            info.compressed_size < info.original_size,
            "compressed {} should be < original {}",
            info.compressed_size,
            info.original_size
        );

        let read_data = store.read_chunk(&info.hash)?;
        assert_eq!(read_data.len(), data.len());

        Ok(())
    }

    #[test]
    fn different_data_different_hash() -> Result<()> {
        let tmp = TempDir::new()?;
        let store = ChunkStore::new(tmp.path().join("chunks"), 3)?;

        let a = store.store_chunk(b"data one")?;
        let b = store.store_chunk(b"data two")?;
        assert_ne!(a.hash, b.hash);

        Ok(())
    }

    #[test]
    fn hash_sharded_path_structure() -> Result<()> {
        let tmp = TempDir::new()?;
        let store = ChunkStore::new(tmp.path().join("chunks"), 3)?;

        let info = store.store_chunk(b"path test")?;
        let expected = tmp
            .path()
            .join("chunks")
            .join(&info.hash[0..2])
            .join(format!("{}.zst", &info.hash[2..]));
        assert!(expected.exists());

        Ok(())
    }

    #[test]
    fn empty_data_chunk() -> Result<()> {
        let tmp = TempDir::new()?;
        let store = ChunkStore::new(tmp.path().join("chunks"), 3)?;

        let info = store.store_chunk(b"")?;
        assert!(info.is_new);
        assert_eq!(info.original_size, 0);

        let read = store.read_chunk(&info.hash)?;
        assert!(read.is_empty());

        Ok(())
    }
}
