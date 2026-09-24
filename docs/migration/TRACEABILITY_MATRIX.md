# 📋 Ma Trận Ánh Xạ & Truy Vết 1:1 (Traceability Matrix)

> **Mục đích:** Đảm bảo 100% tính năng, logic, xử lý lỗi và edge-cases của codebase Kotlin (`legacy/kotlin-compose`) được chuyển dịch chính xác sang Rust (`dev`) mà không bị thất lạc.

---

## 1. Core Downloader Engine (`downloader:core` $\rightarrow$ `crates/flow_core`)

| Phân hệ / Class Kotlin (Legacy) | File / Struct Rust (Mới) | Trọng tâm Logic & Thuật toán cần khớp 100% | Test Coverage | Trạng thái |
| :--- | :--- | :--- | :--- | :---: |
| `PartDownloader.kt`<br>`HttpPartDownloader.kt` | `flow_core::downloader::part`<br>`flow_core::downloader::http_part` | - HTTP 206 Partial Content range requests.<br>- Buffer 64KB I/O streaming.<br>- Dynamic split part khi có luồng tải xong sớm.<br>- Pause, Resume, Cancel an toàn. | `tests/test_part_downloader.rs` | ⏳ Pending |
| `HLSDownloadJob.kt`<br>`HLSResponseInfo.kt` | `flow_core::downloader::hls` | - Parse M3U8 Master / Media Playlists.<br>- Tải song song các media chunks (.ts, .aac, .fmp4).<br>- Tự động nối và ghi tuần tự vào file đích. | `tests/test_hls_downloader.rs` | ⏳ Pending |
| `HttpRetryPolicy.kt` | `flow_core::resilience::backoff` | - Exponential backoff: `1s -> 2s -> 4s -> 8s -> 16s -> 30s`.<br>- Jitter và max retry limits.<br>- Xử lý ngắt kết nối mạng tạm thời. | `tests/test_retry_policy.rs` | ⏳ Pending |
| `SparseFile.kt` | `flow_core::storage::sparse` | - Windows API `FSCTL_SET_SPARSE` qua `windows-sys`.<br>- Cấp phát dung lượng đĩa tức thì (instant allocation) không zero-fill. | `tests/test_sparse_file.rs` | ⏳ Pending |
| `ChecksumUtil.kt` | `flow_core::checksum` | - Streaming Hash: MD5, SHA-1, SHA-256.<br>- Cập nhật digest song song trong quá trình ghi đĩa. | `tests/test_checksum.rs` | ⏳ Pending |
| `DestWriter.kt`<br>`SimpleDownloadDestination.kt` | `flow_core::storage::dest_writer` | - Thread-safe multi-part file writer.<br>- Seek offset và flush an toàn. | `tests/test_dest_writer.rs` | ⏳ Pending |

---

## 2. Queue Manager & Persistence (`downloader:core/db` $\rightarrow$ `crates/flow_core::queue`)

| Phân hệ / Class Kotlin (Legacy) | File / Struct Rust (Mới) | Trọng tâm Logic & Thuật toán cần khớp 100% | Test Coverage | Trạng thái |
| :--- | :--- | :--- | :--- | :---: |
| `QueueManager.kt`<br>`DownloadQueue.kt` | `flow_core::queue::manager`<br>`flow_core::queue::queue_actor` | - Quản lý nhiều hàng đợi tải độc lập.<br>- Giới hạn số file tải đồng thời (`maxConcurrent`).<br>- Actor message passing (`tokio::sync::mpsc`). | `tests/test_queue_manager.rs` | ⏳ Pending |
| `TransactionalFileSaver.kt`<br>`DownloadListFileStorage.kt` | `flow_core::storage::atomic_store` | - Ghi nguyên tử qua file `.tmp` và rename `std::fs::rename`.<br>- JSON Serialization qua `serde_json`.<br>- Chống hỏng dữ liệu khi crash/mất điện. | `tests/test_atomic_store.rs` | ⏳ Pending |
| Schedule times (Auto-start/stop) | `flow_core::queue::scheduler` | - Lập lịch tự động bật/tắt queue theo giờ cấu hình. | `tests/test_scheduler.rs` | ⏳ Pending |

---

## 3. Browser Integration & IPC (`integration:server` $\rightarrow$ `crates/flow_server`)

| API / Endpoint (`REST-API.yml`) | Handler Rust (Axum) | Payload & Contract | Tương thích Extension | Trạng thái |
| :--- | :--- | :--- | :--- | :---: |
| `POST /add` | `routes::add_download` | Request: `[ { link, headers, downloadPage } ]`<br>Response: `200 OK` | ✅ 100% Chrome/Firefox Ext | ⏳ Pending |
| `GET /queues` | `routes::list_queues` | Response: `[ { id: 1, name: "Default" } ]` | ✅ 100% Chrome/Firefox Ext | ⏳ Pending |
| `POST /start-headless-download` | `routes::headless_download` | Request: `{ downloadSource, folder, name, queueId }` | ✅ 100% Chrome/Firefox Ext | ⏳ Pending |
| `SingleInstanceServer.kt` | `flow_server::single_instance` | - Mutex Lock chống mở 2 app.<br>- Chuyển tiếp URL sang instance chính qua local socket. | N/A | ⏳ Pending |

---

## 4. Desktop GUI & Interaction (`desktop:app` $\rightarrow$ `crates/flow_desktop`)

| Màn hình / Component Kotlin | Component Tauri v2 (Svelte/Vue + Tailwind) | Chức năng chi tiết | Trạng thái |
| :--- | :--- | :--- | :---: |
| `HomeScreen.kt` | `src/views/HomeView.svelte` | Danh sách tải file, filter (All, Downloading, Completed, Error), thanh tốc độ tổng. | ⏳ Pending |
| `AddDownloadDialog.kt` | `src/components/AddDownloadModal.svelte` | Dán URL, tự bắt clipboard, chọn folder, chọn queue, chỉnh số thread. | ⏳ Pending |
| `BatchDownloadDialog.kt` | `src/components/BatchDownloadModal.svelte` | Nhập nhiều URL theo mẫu hoặc pattern dải số. | ⏳ Pending |
| `QueueManagerDialog.kt` | `src/components/QueueManagerModal.svelte` | Cấu hình queue, lập lịch giờ chạy, giới hạn tốc độ từng queue. | ⏳ Pending |
| `ChecksumDialog.kt` | `src/components/ChecksumModal.svelte` | So khớp mã băm file tải về với mã hash mong muốn. | ⏳ Pending |
| `SettingsScreen.kt` | `src/views/SettingsView.svelte` | Cài đặt folder mặc định, dark/light theme, proxy, autostart, port integration. | ⏳ Pending |
| `SystemTray.kt` | Tauri System Tray API | Menu tray: Mở app, Pause All, Resume All, Thoát. | ⏳ Pending |
