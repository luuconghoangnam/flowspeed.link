use crate::types::{DownloadStatus, DownloadTask};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct QueueManager {
    tasks: Arc<RwLock<HashMap<String, DownloadTask>>>,
    pub max_concurrent: usize,
}

impl QueueManager {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            max_concurrent,
        }
    }

    /// Thêm task mới vào hàng đợi
    /// Ánh xạ 1:1 từ com.flowspeed.lib.downloader.queue.QueueManager
    pub async fn add_task(&self, task: DownloadTask) {
        let mut map = self.tasks.write().await;
        map.insert(task.id.clone(), task);
    }

    /// Lấy danh sách toàn bộ các task
    pub async fn get_all_tasks(&self) -> Vec<DownloadTask> {
        let map = self.tasks.read().await;
        map.values().cloned().collect()
    }

    /// Cập nhật trạng thái của task
    pub async fn update_status(&self, id: &str, status: DownloadStatus) {
        let mut map = self.tasks.write().await;
        if let Some(task) = map.get_mut(id) {
            task.status = status;
        }
    }
}
