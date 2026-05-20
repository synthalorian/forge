use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};

use crate::config::Config;

struct HashingWriter {
    file: fs::File,
    hasher: Sha256,
}

impl HashingWriter {
    fn new(file: fs::File) -> Self {
        Self {
            file,
            hasher: Sha256::new(),
        }
    }

    fn finalize(self) -> String {
        hex::encode(self.hasher.finalize())
    }
}

impl Write for HashingWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.hasher.update(buf);
        self.file.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

/// Create a compressed archive from a bare git clone.
///
/// Takes a path to a bare git repository, compresses it using zstd at the
/// configured level, and stores the result in the archive directory.
/// Returns the path to the created archive and its SHA-256 hash.
pub fn create_archive(
    cfg: &Config,
    bare_repo_path: &str,
    compression_level: u32,
) -> Result<(String, String)> {
    let repo_path = Path::new(bare_repo_path);
    let repo_name = repo_path
        .file_name()
        .context("Cannot extract repo name from path")?
        .to_str()
        .context("Repo name is not valid UTF-8")?;

    let timestamp = chrono::Local::now().format("%Y-%m-%dT%H-%M-%S");
    let archive_name = format!("{}-{}.tar.zst", repo_name, timestamp);
    let archive_path = cfg.archive_dir.join(&archive_name);

    fs::create_dir_all(&cfg.archive_dir)
        .with_context(|| format!("Failed to create archive dir {}", cfg.archive_dir.display()))?;

    let parent_dir = repo_path
        .parent()
        .context("Repo path has no parent directory")?;

    let mut tar_child = Command::new("tar")
        .args(["-cf", "-", "-C"])
        .arg(parent_dir)
        .arg(repo_name)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to spawn tar command")?;

    let tar_stdout = tar_child
        .stdout
        .take()
        .context("No stdout from tar process")?;

    let output_file = fs::File::create(&archive_path)
        .with_context(|| format!("Failed to create archive file {}", archive_path.display()))?;
    let hashing_writer = HashingWriter::new(output_file);

    let level = compression_level.min(22) as i32;
    let mut encoder =
        zstd::Encoder::new(hashing_writer, level).context("Failed to create zstd encoder")?;

    let mut reader = io::BufReader::with_capacity(64 * 1024, tar_stdout);
    io::copy(&mut reader, &mut encoder)
        .context("Failed to stream tar data through zstd encoder")?;

    let hashing_writer = encoder.finish().context("Failed to finish zstd encoding")?;
    let sha256_hex = hashing_writer.finalize();

    let status = tar_child.wait().context("Failed to wait for tar process")?;
    if !status.success() {
        let _ = fs::remove_file(&archive_path);
        anyhow::bail!("tar command failed with exit code {}", status);
    }

    let archive_str = archive_path
        .to_str()
        .context("Archive path is not valid UTF-8")?
        .to_string();

    Ok((archive_str, sha256_hex))
}

/// Verify an archive's integrity by checking its SHA-256 hash.
pub fn verify_archive(archive_path: &str, expected_sha256: &str) -> Result<bool> {
    let file = fs::File::open(archive_path)
        .with_context(|| format!("Failed to open archive {}", archive_path))?;

    let mut reader = io::BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];

    loop {
        let n = reader
            .read(&mut buf)
            .with_context(|| format!("Failed to read archive {}", archive_path))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }

    let actual = hex::encode(hasher.finalize());
    Ok(actual.eq_ignore_ascii_case(expected_sha256))
}

/// Extract an archive to a target directory.
pub fn extract_archive(archive_path: &str, target_dir: &str) -> Result<()> {
    let file = fs::File::open(archive_path)
        .with_context(|| format!("Failed to open archive {}", archive_path))?;
    let reader = io::BufReader::new(file);

    let mut decoder = zstd::Decoder::new(reader)
        .with_context(|| format!("Failed to create zstd decoder for {}", archive_path))?;

    fs::create_dir_all(target_dir)
        .with_context(|| format!("Failed to create target directory {}", target_dir))?;

    let mut tar_child = Command::new("tar")
        .args(["-xf", "-", "-C"])
        .arg(target_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to spawn tar command for extraction")?;

    {
        let mut tar_stdin = tar_child.stdin.take().context("No stdin for tar process")?;
        io::copy(&mut decoder, &mut tar_stdin)
            .context("Failed to stream decompressed data to tar")?;
    }

    let status = tar_child.wait().context("Failed to wait for tar process")?;
    if !status.success() {
        anyhow::bail!("tar extraction failed with exit code {}", status);
    }

    Ok(())
}

pub struct ChunkMeta {
    pub hash: String,
    pub original_size: u64,
    pub compressed_size: u64,
}

pub struct DedupArchiveResult {
    pub manifest_path: String,
    pub manifest_sha256: String,
    pub total_chunks: u64,
    pub new_chunks: u64,
    pub original_size: u64,
    pub compressed_size: u64,
    pub chunk_hashes: Vec<String>,
    pub chunk_details: Vec<ChunkMeta>,
}

pub fn create_dedup_archive(
    cfg: &Config,
    bare_repo_path: &str,
    compression_level: u32,
) -> Result<DedupArchiveResult> {
    use crate::chunkstore::{ChunkStore, DEFAULT_CHUNK_SIZE};

    let repo_path = Path::new(bare_repo_path);
    let repo_name = repo_path
        .file_name()
        .context("Cannot extract repo name from path")?
        .to_str()
        .context("Repo name is not valid UTF-8")?;

    let chunks_dir = cfg.archive_dir.join("chunks");
    let chunk_store = ChunkStore::new(chunks_dir, compression_level as i32)?;

    let parent_dir = repo_path
        .parent()
        .context("Repo path has no parent directory")?;

    let mut tar_child = Command::new("tar")
        .args(["-cf", "-", "-C"])
        .arg(parent_dir)
        .arg(repo_name)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to spawn tar command")?;

    let tar_stdout = tar_child
        .stdout
        .take()
        .context("No stdout from tar process")?;
    let mut reader = io::BufReader::with_capacity(64 * 1024, tar_stdout);

    let mut chunk_hashes: Vec<String> = Vec::new();
    let mut chunk_details: Vec<ChunkMeta> = Vec::new();
    let mut total_chunks: u64 = 0;
    let mut new_chunks: u64 = 0;
    let mut original_size: u64 = 0;
    let mut compressed_size: u64 = 0;

    let mut buf = vec![0u8; DEFAULT_CHUNK_SIZE];
    loop {
        let mut chunk_data = Vec::with_capacity(DEFAULT_CHUNK_SIZE);
        loop {
            let n = reader
                .read(&mut buf)
                .context("Failed to read from tar stream")?;
            if n == 0 {
                break;
            }
            chunk_data.extend_from_slice(&buf[..n]);
            if chunk_data.len() >= DEFAULT_CHUNK_SIZE {
                break;
            }
        }
        if chunk_data.is_empty() {
            break;
        }

        let info = chunk_store.store_chunk(&chunk_data)?;
        original_size += chunk_data.len() as u64;
        if info.is_new {
            new_chunks += 1;
            compressed_size += info.compressed_size as u64;
        }
        total_chunks += 1;
        chunk_hashes.push(info.hash.clone());
        chunk_details.push(ChunkMeta {
            hash: info.hash,
            original_size: info.original_size as u64,
            compressed_size: info.compressed_size as u64,
        });
    }

    let status = tar_child.wait().context("Failed to wait for tar process")?;
    if !status.success() {
        anyhow::bail!("tar command failed with exit code {}", status);
    }

    let timestamp = chrono::Local::now().format("%Y-%m-%dT%H-%M-%S");
    let manifest_name = format!("{}-{}.manifest.json", repo_name, timestamp);
    let manifest_path = cfg.archive_dir.join(&manifest_name);

    let manifest = serde_json::json!({
        "chunk_hashes": chunk_hashes,
        "compression_level": compression_level,
        "original_size": original_size,
        "compressed_size": compressed_size,
        "created_at": chrono::Utc::now().to_rfc3339(),
    });

    let manifest_str =
        serde_json::to_string_pretty(&manifest).context("Failed to serialize manifest")?;
    fs::write(&manifest_path, &manifest_str)
        .with_context(|| format!("Failed to write manifest to {}", manifest_path.display()))?;

    let manifest_bytes = fs::read(&manifest_path)
        .with_context(|| format!("Failed to read manifest {}", manifest_path.display()))?;
    let mut hasher = Sha256::new();
    hasher.update(&manifest_bytes);
    let manifest_sha256 = hex::encode(hasher.finalize());

    Ok(DedupArchiveResult {
        manifest_path: manifest_path
            .to_str()
            .context("Manifest path is not valid UTF-8")?
            .to_string(),
        manifest_sha256,
        total_chunks,
        new_chunks,
        original_size,
        compressed_size,
        chunk_hashes,
        chunk_details,
    })
}

pub fn extract_dedup_archive(cfg: &Config, manifest_path: &str, target_dir: &str) -> Result<()> {
    use crate::chunkstore::ChunkStore;

    let manifest_content = fs::read_to_string(manifest_path)
        .with_context(|| format!("Failed to read manifest {}", manifest_path))?;
    let manifest: serde_json::Value = serde_json::from_str(&manifest_content)
        .with_context(|| format!("Failed to parse manifest {}", manifest_path))?;

    let chunk_hashes: Vec<String> = manifest["chunk_hashes"]
        .as_array()
        .context("Manifest missing chunk_hashes array")?
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();

    let chunks_dir = cfg.archive_dir.join("chunks");
    let chunk_store = ChunkStore::new(chunks_dir, 3)?;

    fs::create_dir_all(target_dir)
        .with_context(|| format!("Failed to create target directory {}", target_dir))?;

    let mut tar_child = Command::new("tar")
        .args(["-xf", "-", "-C"])
        .arg(target_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to spawn tar command for extraction")?;

    {
        let mut tar_stdin = tar_child.stdin.take().context("No stdin for tar process")?;
        for hash in &chunk_hashes {
            let data = chunk_store
                .read_chunk(hash)
                .with_context(|| format!("Failed to read chunk {}", hash))?;
            tar_stdin
                .write_all(&data)
                .context("Failed to write chunk to tar stdin")?;
        }
    }

    let status = tar_child.wait().context("Failed to wait for tar process")?;
    if !status.success() {
        anyhow::bail!("tar extraction failed with exit code {}", status);
    }

    Ok(())
}
