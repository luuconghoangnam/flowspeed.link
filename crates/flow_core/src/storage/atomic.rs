use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Write};
use std::path::Path;

pub struct AtomicJsonStorage;

impl AtomicJsonStorage {
    /// Ghi dữ liệu nguyên tử (atomic write) qua file tạm .tmp rồi rename
    /// Ánh xạ 1:1 từ com.flowspeed.lib.downloader.db.TransactionalFileSaver
    pub fn save_atomic<T: Serialize>(path: impl AsRef<Path>, data: &T) -> io::Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let tmp_path = path.with_extension("tmp");
        {
            let file = File::create(&tmp_path)?;
            let mut writer = BufWriter::new(file);
            serde_json::to_writer_pretty(&mut writer, data)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            writer.flush()?;
        }

        fs::rename(tmp_path, path)?;
        Ok(())
    }

    /// Đọc dữ liệu JSON an toàn
    pub fn load<T: DeserializeOwned>(path: impl AsRef<Path>) -> io::Result<Option<T>> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(None);
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let data = serde_json::from_reader(reader)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(Some(data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_atomic_save_and_load() -> io::Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("tasks.json");

        #[derive(Serialize, serde::Deserialize, PartialEq, Debug)]
        struct SampleData {
            name: String,
            count: u32,
        }

        let original = SampleData {
            name: "test_task".to_string(),
            count: 42,
        };

        AtomicJsonStorage::save_atomic(&file_path, &original)?;
        let loaded: Option<SampleData> = AtomicJsonStorage::load(&file_path)?;

        assert_eq!(loaded, Some(original));
        Ok(())
    }
}
