# 📊 Tiến Độ Chuyển Dịch (Migration Progress Tracker)

## 📌 Tổng Quan Tiến Độ

- **Ngôn ngữ đích:** Rust (1.80+) + Tauri v2
- **Frontend đích:** Svelte 5 + TailwindCSS
- **Trạng thái hiện tại:** **Giai đoạn 3 & 4 - Hoàn thiện Tính năng Desktop & Tích hợp Toàn diện**
- **Test Coverage:** 15/15 unit tests pass 100% (`cargo test --workspace`)

---

## 🎯 Danh Sách Hạng Mục Công Việc

### Giai đoạn 1: Rust Core Engine (`crates/flow_core`)
- [x] Khởi tạo Cargo Workspace (`Cargo.toml`, `crates/flow_core`).
- [x] Module `flow_core::types` (DownloadTask, PartState, TaskStatus, DownloadProgress, AppSettings, FileCategory, PerHostRule).
- [x] Module `flow_core::resilience::backoff` (Exponential Backoff + Jitter).
- [x] Module `flow_core::checksum` (Streaming MD5, SHA-1, SHA-256).
- [x] Module `flow_core::storage::sparse` (Windows NTFS `FSCTL_SET_SPARSE`).
- [x] Module `flow_core::storage::atomic` (Atomic file JSON storage).
- [x] Module `flow_core::downloader::coordinator` (`HttpDownloadCoordinator`, `HttpPartDownloader`, `UrlProber`, `SpeedMeter`).
- [x] Module `flow_core::downloader::hls` (M3U8 Master/Media Parser & Parallel TS downloader).
- [x] Module `flow_core::queue::manager` (Actor-based Concurrency Manager & Scheduler).
- [x] Unit Test Suite (`cargo test` 15/15 pass 100%).

### Giai đoạn 2: Integration Server & IPC (`crates/flow_server`)
- [x] Module `flow_server::routes` (Axum REST API port 15151: `/add`, `/queues`, `/start-headless-download`, `/`).
- [x] Single Instance Mutex (`SingleInstance` lock).
- [x] CORS & Browser Extension Integration Protocol.
- [x] Nhúng trực tiếp Axum server vào `flow_desktop` background thread để nhận link từ Extension thời gian thực.

### Giai đoạn 3: Tauri v2 Desktop GUI (`crates/flow_desktop`)
- [x] Scaffolding Tauri v2 App với Svelte 5 + TailwindCSS (`postcss.config.js`).
- [x] Tauri Permissions & ACL (`capabilities/default.json` - `core:event:allow-listen`, `allow-emit`).
- [x] Tauri Commands (`start_download`, `cancel_download`, `open_in_folder`, `calculate_checksum`, `get_app_settings`, `save_app_settings`).
- [x] Tauri Events Streaming (`download-progress`, `download-completed`, `download-error`).
- [x] Màn hình chính Home (Downloads List, Realtime speed, ETA, Filter, Open Folder).
- [x] Modal Thêm liên kết tải (Single URL & Test links).
- [x] Modal Tải hàng loạt (Batch Download).
- [x] Modal So khớp mã băm (Checksum Tool SHA-256 / SHA-1 / MD5).
- [x] Modal Quản lý Hàng đợi (Queue Manager & Lập lịch ban đêm).
- [x] Modal Cài đặt tổng quan (Settings: Paths, Threads, Concurrency, Speed Limit, Port).
- [x] Tự động Phân loại tệp theo Danh mục (File Categories: Video, Audio, Archives, Documents, Programs).
- [x] Quy tắc kết nối theo Tên miền (Per-Host Rules: Max connections, Custom headers, Domain proxy).
- [x] Hành động Nguồn khi tải xong (Power Action: Shutdown, Sleep, Hibernate + Cửa sổ đếm ngược 30s).
- [x] System Tray Menu (Open, Hide, Quit) & Windows Toast Notifications.


### Giai đoạn 4: Đóng Gói & Tối Ưu Hóa
- [x] Cấu hình Release Profile (`lto = true`, `opt-level = 3`, `strip = true`, `codegen-units = 1`).
- [ ] Build Windows Installer `.msi` / `.exe`.
- [ ] Benchmark kiểm thử: RAM <30MB, CPU <1% ở 1Gbps, Startup <30ms.
