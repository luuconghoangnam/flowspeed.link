use md5::{Digest, Md5};
use sha1::Sha1;
use sha2::Sha256;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChecksumAlgorithm {
    Md5,
    Sha1,
    Sha256,
}

pub struct ChecksumUtil;

impl ChecksumUtil {
    /// Tính toán mã băm của file dạng streaming không làm tràn RAM
    /// Ánh xạ 1:1 từ com.flowspeed.lib.downloader.checksum.ChecksumUtil
    pub fn calculate_file_checksum(
        path: impl AsRef<Path>,
        algo: ChecksumAlgorithm,
    ) -> io::Result<String> {
        let mut file = File::open(path)?;
        let mut buffer = [0u8; 65536]; // 64KB buffer

        match algo {
            ChecksumAlgorithm::Md5 => {
                let mut hasher = Md5::new();
                loop {
                    let n = file.read(&mut buffer)?;
                    if n == 0 {
                        break;
                    }
                    hasher.update(&buffer[..n]);
                }
                Ok(hex::encode(hasher.finalize()))
            }
            ChecksumAlgorithm::Sha1 => {
                let mut hasher = Sha1::new();
                loop {
                    let n = file.read(&mut buffer)?;
                    if n == 0 {
                        break;
                    }
                    hasher.update(&buffer[..n]);
                }
                Ok(hex::encode(hasher.finalize()))
            }
            ChecksumAlgorithm::Sha256 => {
                let mut hasher = Sha256::new();
                loop {
                    let n = file.read(&mut buffer)?;
                    if n == 0 {
                        break;
                    }
                    hasher.update(&buffer[..n]);
                }
                Ok(hex::encode(hasher.finalize()))
            }
        }
    }
}

mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes
            .as_ref()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_file_checksum_sha256() -> io::Result<()> {
        let mut file = NamedTempFile::new()?;
        file.write_all(b"Flow Speed Link Rust Engine")?;
        file.flush()?;

        let hash = ChecksumUtil::calculate_file_checksum(file.path(), ChecksumAlgorithm::Sha256)?;
        assert_eq!(
            hash,
            "b322769fbd2b8a8fb868d459a3f41e6424b10124428be3c84f11af38390bb173"
        );
        Ok(())
    }
}
