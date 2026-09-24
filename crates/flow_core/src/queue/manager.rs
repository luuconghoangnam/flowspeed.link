use crate::storage::AtomicJsonStorage;
use crate::types::{DownloadQueue, DownloadStatus, DownloadTask};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct QueueManager {
    queues: Arc<RwLock<HashMap<i64, DownloadQueue>>>,
    tasks: Arc<RwLock<HashMap<String, DownloadTask>>>,
    storage_path: Option<PathBuf>,
}

impl QueueManager {
    pub fn new() -> Self {
        let mut queues = HashMap::new();
        let default_queue = DownloadQueue::default();
        queues.insert(default_queue.id, default_queue);

        Self {
            queues: Arc::new(RwLock::new(queues)),
            tasks: Arc::new(RwLock::new(HashMap::new())),
            storage_path: None,
        }
    }

    pub fn with_storage(storage_path: impl AsRef<Path>) -> Self {
        let mut manager = Self::new();
        manager.storage_path = Some(storage_path.as_ref().to_path_buf());
        manager
    }

    /// Thêm hoặc cập nhật hàng đợi
    pub async fn create_queue(&self, name: String, max_concurrent: usize) -> DownloadQueue {
        let mut q_map = self.queues.write().await;
        let next_id = (q_map.keys().max().copied().unwrap_or(0)) + 1;
        let queue = DownloadQueue {
            id: next_id,
            name,
            max_concurrent: max_concurrent.max(1),
            speed_limit_kbps: None,
            auto_start_time: None,
            auto_stop_time: None,
            is_active: true,
        };
        q_map.insert(next_id, queue.clone());
        queue
    }

    /// Lấy danh sách toàn bộ hàng đợi
    pub async fn get_all_queues(&self) -> Vec<DownloadQueue> {
        let q_map = self.queues.read().await;
        q_map.values().cloned().collect()
    }

    /// Xóa một hàng đợi (không cho phép xóa hàng đợi mặc định id=1)
    pub async fn delete_queue(&self, queue_id: i64) -> bool {
        if queue_id == 1 {
            return false;
        }
        let mut q_map = self.queues.write().await;
        q_map.remove(&queue_id).is_some()
    }

    /// Thêm một task mới vào hàng đợi
    /// Ánh xạ 1:1 từ com.flowspeed.lib.downloader.queue.QueueManager
    pub async fn add_task(&self, mut task: DownloadTask) {
        if task.queue_id.is_none() {
            task.queue_id = Some(1); // Default queue
        }
        let mut map = self.tasks.write().await;
        map.insert(task.id.clone(), task);
    }

    /// Lấy toàn bộ danh sách task
    pub async fn get_all_tasks(&self) -> Vec<DownloadTask> {
        let map = self.tasks.read().await;
        map.values().cloned().collect()
    }

    /// Lấy thông tin một task theo id
    pub async fn get_task(&self, id: &str) -> Option<DownloadTask> {
        let map = self.tasks.read().await;
        map.get(id).cloned()
    }

    /// Đếm số task đang chạy trong một hàng đợi cụ thể
    pub async fn active_task_count(&self, queue_id: i64) -> usize {
        let map = self.tasks.read().await;
        map.values()
            .filter(|t| t.queue_id == Some(queue_id) && t.status == DownloadStatus::Downloading)
            .count()
    }

    /// Lấy task tiếp theo có thể khởi chạy trong hàng đợi dựa trên giới hạn đồng thời (Concurrency Limit)
    pub async fn get_next_runnable_task(&self, queue_id: i64) -> Option<DownloadTask> {
        let q_map = self.queues.read().await;
        let queue = q_map.get(&queue_id)?;
        if !queue.is_active {
            return None;
        }

        let running_count = self.active_task_count(queue_id).await;
        if running_count >= queue.max_concurrent {
            return None;
        }

        let map = self.tasks.read().await;
        map.values()
            .filter(|t| t.queue_id == Some(queue_id) && t.status == DownloadStatus::Queued)
            .min_by_key(|t| t.created_at)
            .cloned()
    }

    /// Cập nhật trạng thái của task
    pub async fn update_task_status(&self, id: &str, status: DownloadStatus) {
        let mut map = self.tasks.write().await;
        if let Some(task) = map.get_mut(id) {
            task.status = status;
            task.updated_at = chrono::Utc::now();
        }
    }

    /// Xóa một task khỏi hàng đợi
    pub async fn remove_task(&self, id: &str) -> Option<DownloadTask> {
        let mut map = self.tasks.write().await;
        map.remove(id)
    }

    /// Lưu toàn bộ trạng thái task xuống ổ đĩa an toàn
    pub async fn save_to_disk(&self) -> std::io::Result<()> {
        if let Some(ref path) = self.storage_path {
            let tasks: Vec<DownloadTask> = self.get_all_tasks().await;
            AtomicJsonStorage::save_atomic(path, &tasks)?;
        }
        Ok(())
    }

    /// Tải lại trạng thái task từ ổ đĩa
    pub async fn load_from_disk(&self) -> std::io::Result<()> {
        if let Some(ref path) = self.storage_path {
            if let Some(tasks) = AtomicJsonStorage::load::<Vec<DownloadTask>>(path)? {
                let mut map = self.tasks.write().await;
                map.clear();
                for task in tasks {
                    map.insert(task.id.clone(), task);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::DownloadType;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_queue_lifecycle_and_concurrency() {
        let manager = QueueManager::new();

        // Kiểm tra tạo queue
        let q2 = manager.create_queue("Media Queue".to_string(), 2).await;
        assert_eq!(q2.name, "Media Queue");
        assert_eq!(q2.max_concurrent, 2);

        // Tạo 3 task ở trạng thái Queued
        for i in 1..=3 {
            let task = DownloadTask {
                id: format!("task-{}", i),
                url: format!("https://example.com/file{}.zip", i),
                filename: format!("file{}.zip", i),
                save_path: "/downloads".to_string(),
                total_bytes: Some(1024),
                downloaded_bytes: 0,
                status: DownloadStatus::Queued,
                download_type: DownloadType::HttpRange,
                parts: vec![],
                headers: HashMap::new(),
                queue_id: Some(q2.id),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };
            manager.add_task(task).await;
        }

        // Lấy task đầu tiên để chạy
        let next1 = manager.get_next_runnable_task(q2.id).await;
        assert!(next1.is_some());
        manager
            .update_task_status(&next1.unwrap().id, DownloadStatus::Downloading)
            .await;

        // Lấy task thứ 2 để chạy
        let next2 = manager.get_next_runnable_task(q2.id).await;
        assert!(next2.is_some());
        manager
            .update_task_status(&next2.unwrap().id, DownloadStatus::Downloading)
            .await;

        // Queue giới hạn max_concurrent = 2 -> Lúc này không thể lấy thêm task thứ 3
        let next3 = manager.get_next_runnable_task(q2.id).await;
        assert!(next3.is_none());

        // Khi 1 task hoàn thành -> Task thứ 3 có thể khởi chạy
        manager
            .update_task_status("task-1", DownloadStatus::Completed)
            .await;
        let next_after_complete = manager.get_next_runnable_task(q2.id).await;
        assert!(next_after_complete.is_some());
        assert_eq!(next_after_complete.unwrap().id, "task-3");
    }

    #[tokio::test]
    async fn test_queue_persistence_save_and_load() {
        let dir = tempdir().unwrap();
        let db_file = dir.path().join("download_queue.json");

        let manager = QueueManager::with_storage(&db_file);
        let task = DownloadTask {
            id: "persist-1".to_string(),
            url: "https://example.com/test.iso".to_string(),
            filename: "test.iso".to_string(),
            save_path: "/downloads/test.iso".to_string(),
            total_bytes: Some(50000),
            downloaded_bytes: 12000,
            status: DownloadStatus::Downloading,
            download_type: DownloadType::HttpRange,
            parts: vec![],
            headers: HashMap::new(),
            queue_id: Some(1),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        manager.add_task(task).await;
        manager.save_to_disk().await.unwrap();
        assert!(db_file.exists());

        // Tạo instance mới và tải lại từ file
        let new_manager = QueueManager::with_storage(&db_file);
        new_manager.load_from_disk().await.unwrap();
        let tasks = new_manager.get_all_tasks().await;
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].id, "persist-1");
        assert_eq!(tasks[0].downloaded_bytes, 12000);
    }
}

