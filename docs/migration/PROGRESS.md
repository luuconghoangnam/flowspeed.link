# 📊 Tiến Độ Chuyển Dịch (Migration Progress Tracker)

## 📌 Tổng Quan Tiến Độ

- **Ngôn ngữ đích:** Rust (1.80+) + Tauri v2
- **Frontend đích:** Svelte 5 (hoặc Vue 3) + TailwindCSS
- **Trạng thái hiện tại:** **Giai đoạn 1 - Khởi tạo Workspace & Core Engine**

---

## 🎯 Danh Sách Hạng Mục Công Việc

### Giai đoạn 1: Rust Core Engine (`crates/flow_core`)
- [ ] Khởi tạo Cargo Workspace (`Cargo.toml`, `crates/flow_core`).
- [ ] Module `flow_core::types` (DownloadTask, PartState, TaskStatus, DownloadProgress).
- [ ] Module `flow_core::resilience::backoff` (Exponential Backoff + Jitter).
- [ ] Module `flow_core::checksum` (Streaming MD5, SHA-1, SHA-256).
- [ ] Module `flow_core::storage::sparse` (Windows NTFS `FSCTL_SET_SPARSE`).
- [ ] Module `flow_core::storage::atomic` (Atomic file JSON storage).
- [ ] Module `flow_core::downloader::range` (HTTP Range, Multi-part downloader, Dynamic split).
- [ ] Module `flow_core::downloader::hls` (M3U8 Parser & Parallel TS downloader).
- [ ] Module `flow_core::queue::manager` (Actor-based Concurrency Manager & Scheduler).
- [ ] Unit Test Suite (`cargo test` pass 100%).

### Giai đoạn 2: Integration Server & IPC (`crates/flow_server`)
- [ ] Module `flow_server::routes` (Axum REST API port 15151: `/add`, `/queues`, `/start-headless-download`).
- [ ] Single Instance Mutex (`single_instance` crate).
- [ ] CORS & Browser Extension Integration Test.

### Giai đoạn 3: Tauri v2 Desktop GUI (`crates/flow_desktop`)
- [ ] Scaffolding Tauri v2 App với Svelte/Vue + TailwindCSS.
- [ ] Tauri Commands (`start_download`, `pause_download`, `get_tasks`, `save_settings`).
- [ ] Tauri Events Streaming (`download-progress`, `speed-updated`).
- [ ] Giao diện Home, Add Download Modal, Settings, Queue Modal.
- [ ] System Tray, OS Notifications, Autostart.

### Giai đoạn 4: Đóng Gói & Tối Ưu Hóa
- [ ] Cấu hình Release Profile (`lto = true`, `opt-level = 3`, `strip = true`).
- [ ] Build Windows Installer `.msi` / `.exe`.
- [ ] Benchmark kiểm thử: RAM <30MB, CPU <1% ở 1Gbps, Startup <30ms.
