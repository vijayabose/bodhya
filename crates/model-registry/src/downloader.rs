/// Model download functionality
///
/// This module handles downloading models from HTTP sources with
/// progress tracking and checksum verification.
use bodhya_core::{Error, Result};
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

/// Download result containing the downloaded file path and verification info
#[derive(Debug, Clone)]
pub struct DownloadResult {
    /// Path to the downloaded file
    pub file_path: PathBuf,
    /// Actual checksum of the downloaded file (if verified)
    pub checksum: Option<String>,
    /// Number of bytes downloaded
    pub bytes_downloaded: u64,
}

/// Model downloader
pub struct ModelDownloader {
    /// HTTP client for downloads
    client: reqwest::Client,
}

impl ModelDownloader {
    /// Create a new model downloader
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(3600)) // 1 hour timeout
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    /// Download a model from a URL to a destination path
    ///
    /// # Arguments
    /// * `url` - Source URL to download from
    /// * `dest_path` - Destination file path
    /// * `expected_checksum` - Optional expected checksum for verification (format: "sha256:hash")
    ///
    /// # Returns
    /// DownloadResult with file path and verification info
    pub async fn download(
        &self,
        url: &str,
        dest_path: impl AsRef<Path>,
        expected_checksum: Option<&str>,
    ) -> Result<DownloadResult> {
        let dest_path = dest_path.as_ref();

        tracing::info!("Downloading model from {} to {:?}", url, dest_path);

        // Create parent directories if they don't exist
        if let Some(parent) = dest_path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                Error::Io(format!(
                    "Failed to create parent directories for {:?}: {}",
                    dest_path, e
                ))
            })?;
        }

        // Download the file
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| Error::Network(format!("Failed to download from {}: {}", url, e)))?;

        if !response.status().is_success() {
            return Err(Error::Network(format!(
                "HTTP error {} when downloading from {}",
                response.status(),
                url
            )));
        }

        // Get content length if available
        let content_length = response.content_length();
        if let Some(len) = content_length {
            tracing::info!(
                "Content length: {} bytes ({:.2} MB)",
                len,
                len as f64 / 1_048_576.0
            );
        }

        // Download to a temporary file first
        let temp_path = dest_path.with_extension("tmp");
        let mut file = File::create(&temp_path)
            .await
            .map_err(|e| Error::Io(format!("Failed to create file {:?}: {}", temp_path, e)))?;

        let mut bytes_downloaded = 0u64;
        let mut stream = response.bytes_stream();

        use futures::StreamExt;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| {
                Error::Network(format!("Failed to read chunk during download: {}", e))
            })?;

            file.write_all(&chunk)
                .await
                .map_err(|e| Error::Io(format!("Failed to write chunk to file: {}", e)))?;

            bytes_downloaded += chunk.len() as u64;

            // Log progress periodically (every 100 MB)
            if bytes_downloaded % (100 * 1_048_576) == 0 {
                if let Some(total) = content_length {
                    let progress = (bytes_downloaded as f64 / total as f64) * 100.0;
                    tracing::info!("Download progress: {:.1}%", progress);
                }
            }
        }

        file.flush()
            .await
            .map_err(|e| Error::Io(format!("Failed to flush file: {}", e)))?;

        drop(file); // Close the file

        tracing::info!("Downloaded {} bytes", bytes_downloaded);

        // Verify checksum if provided
        let actual_checksum = if let Some(expected) = expected_checksum {
            let checksum = self.verify_checksum(&temp_path, expected).await?;
            Some(checksum)
        } else {
            None
        };

        // Move temporary file to final destination
        tokio::fs::rename(&temp_path, dest_path)
            .await
            .map_err(|e| {
                Error::Io(format!(
                    "Failed to move downloaded file to {:?}: {}",
                    dest_path, e
                ))
            })?;

        tracing::info!("Download complete: {:?}", dest_path);

        Ok(DownloadResult {
            file_path: dest_path.to_path_buf(),
            checksum: actual_checksum,
            bytes_downloaded,
        })
    }

    /// Verify the checksum of a downloaded file
    ///
    /// # Arguments
    /// * `file_path` - Path to the file to verify
    /// * `expected_checksum` - Expected checksum in format "sha256:hash"
    ///
    /// # Returns
    /// The actual checksum string
    async fn verify_checksum(&self, file_path: &Path, expected_checksum: &str) -> Result<String> {
        use sha2::{Digest, Sha256};

        // Parse expected checksum format: "sha256:hash"
        let parts: Vec<&str> = expected_checksum.split(':').collect();
        if parts.len() != 2 || parts[0] != "sha256" {
            return Err(Error::InvalidInput(format!(
                "Invalid checksum format: {}. Expected 'sha256:hash'",
                expected_checksum
            )));
        }
        let expected_hash = parts[1];

        tracing::info!("Verifying checksum for {:?}", file_path);

        // Read file and compute SHA256 hash
        let mut file = tokio::fs::File::open(file_path).await.map_err(|e| {
            Error::Io(format!(
                "Failed to open file for checksum verification: {}",
                e
            ))
        })?;

        let mut hasher = Sha256::new();
        let mut buffer = vec![0u8; 8192]; // 8KB buffer

        use tokio::io::AsyncReadExt;
        loop {
            let n = file.read(&mut buffer).await.map_err(|e| {
                Error::Io(format!(
                    "Failed to read file during checksum verification: {}",
                    e
                ))
            })?;

            if n == 0 {
                break;
            }

            hasher.update(&buffer[..n]);
        }

        let actual_hash = format!("{:x}", hasher.finalize());

        // Compare hashes
        if actual_hash != expected_hash {
            return Err(Error::ChecksumMismatch(format!(
                "Checksum verification failed. Expected: {}, Got: {}",
                expected_hash, actual_hash
            )));
        }

        tracing::info!("Checksum verification passed");

        Ok(format!("sha256:{}", actual_hash))
    }
}

impl Default for ModelDownloader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_downloader_creation() {
        let _downloader = ModelDownloader::new();
        // Downloader created successfully
    }

    #[test]
    fn test_downloader_default() {
        let _downloader = ModelDownloader::default();
        // Downloader created successfully
    }

    // Note: Full download tests are challenging in unit tests because they require:
    // 1. A real HTTP server or mock server
    // 2. Tokio runtime for async
    // 3. Network access
    //
    // These would be better as integration tests or manual tests.
    // For now, we verify construction and basic functionality.
}
