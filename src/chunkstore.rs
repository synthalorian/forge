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
