# 📋 Ma Trận Ánh Xạ & Truy Vết 1:1 (Traceability Matrix)

> **Mục đích:** Đảm bảo 100% tính năng, logic, xử lý lỗi và edge-cases của codebase Kotlin (`legacy/kotlin-compose`) được chuyển dịch chính xác sang Rust (`dev`) mà không bị thất lạc, đồng thời bảo vệ các thành phần tập trung (Extension, Landing Page).

---

## 1. Core Downloader Engine (`downloader:core` $\rightarrow$ `crates/flow_core`)

| Phân hệ / Class Kotlin (Legacy) | File / Struct Rust (Mới) | Trọng tâm Logic & Thuật toán cần khớp 100% | Test Coverage | Trạng thái |
| :--- | :--- | :--- | :--- | :---: |
| `HttpDownloadJob.kt`<br>`PartDownloader.kt`<br>`HttpPartDownloader.kt` | `flow_core::downloader::coordinator`<br>`flow_core::downloader::part`<br>`flow_core::downloader::probe`<br>`flow_core::downloader::speed` | - Thăm dò HEAD/GET 0-0 metadata URL.<br>- HTTP 206 Range requests.<br>- Slicing N parts song song & sparse file.<br>- SpeedMeter sliding window & ETA.<br>- Buffer 64KB I/O streaming. | `test_multi_part_download_coordinator`<br>`test_eta_calculation`<br>`test_format_speed`<br>`test_parse_content_disposition` | ✅ Completed |
| `HLSDownloadJob.kt`<br>`HLSResponseInfo.kt` | `flow_core::downloader::hls` | - Parse M3U8 Master / Media Playlists.<br>- Tự động chọn bitrate cao nhất từ Master playlist.<br>- Tải song song các media chunks (.ts, .aac, .fmp4).<br>- Tự động nối và ghi tuần tự vào file đích. | `test_hls_stream_downloader` | ✅ Completed |
| `HttpRetryPolicy.kt` | `flow_core::resilience::backoff` | - Exponential backoff: `1s -> 2s -> 4s -> 8s -> 16s -> 30s`.<br>- Jitter và max retry limits.<br>- Xử lý ngắt kết nối mạng tạm thời. | `test_exponential_backoff_calculation` | ✅ Completed |
| `SparseFile.kt` | `flow_core::storage::sparse` | - Windows API `FSCTL_SET_SPARSE` qua `windows-sys`.<br>- Cấp phát dung lượng đĩa tức thì (instant allocation) không zero-fill. | `test_multi_part_download_coordinator` | ✅ Completed |
| `ChecksumUtil.kt` | `flow_core::checksum` | - Streaming Hash: MD5, SHA-1, SHA-256.<br>- Cập nhật digest song song trong quá trình ghi đĩa. | `test_file_checksum_sha256` | ✅ Completed |
| `DestWriter.kt`<br>`SimpleDownloadDestination.kt` | `flow_core::storage::dest_writer` | - Thread-safe multi-part file writer.<br>- Seek offset và flush an toàn. | `test_multi_part_download_coordinator` | ✅ Completed |

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
| `GET /` | `routes::handle_health_check` | Health check endpoint kiểm tra app đang chạy | ✅ 100% `background.js` | ⏳ Pending |
| `POST /add` | `routes::handle_add_downloads` | Request: `{ items: [...], options: {...} }` hoặc `[ { link, ... } ]` | ✅ 100% `background.js` | ⏳ Pending |
| `GET /queues` | `routes::handle_get_queues` | Response: `[ { id: 1, name: "Default" } ]` | ✅ 100% Chrome/Firefox Ext | ⏳ Pending |
| `POST /start-headless-download` | `routes::handle_headless_download`| Request: `{ downloadSource, folder, name, queueId }` | ✅ 100% Chrome/Firefox Ext | ⏳ Pending |
| `SingleInstanceServer.kt` | `flow_server::single_instance` | - Mutex Lock chống mở 2 app.<br>- Chuyển tiếp URL sang instance chính qua local socket. | N/A | ⏳ Pending |

---

## 4. Desktop GUI & Interaction (`desktop:app` $\rightarrow$ `crates/flow_desktop`)

| Màn hình / Component Kotlin | Component Tauri v2 (Svelte/Vue + Tailwind) | Chức năng chi tiết | Trạng thái |
| :--- | :--- | :--- | :---: |
| `HomeScreen.kt` | `src/App.svelte` | Danh sách tải file, filter (All, Downloading, Completed, Error), thanh tốc độ tổng, tiến trình gradient mượt, mở thư mục File Explorer. | ✅ Completed |
| `AddDownloadDialog.kt` | `src/App.svelte` (Add Modal) | Dán URL, tùy chỉnh số threads tải đa luồng, link test mẫu Cloudflare/GitHub, gọi invoke `start_download`. | ✅ Completed |
| `BatchDownloadDialog.kt` | `src/components/BatchDownloadModal.svelte` | Nhập nhiều URL theo danh sách dòng, tùy chỉnh số luồng và thêm đồng loạt. | ✅ Completed |
| `QueueManagerDialog.kt` | `src/components/QueueManagerModal.svelte` | Cấu hình queue, lập lịch giờ chạy, giới hạn tốc độ từng queue. | ⏳ Pending |
| `ChecksumDialog.kt` | `src/components/ChecksumModal.svelte` | So khớp mã băm SHA-256 / SHA-1 / MD5 file tải về với mã hash mong muốn. | ✅ Completed |
| `SettingsScreen.kt` | `src/components/SettingsModal.svelte` | Cài đặt folder mặc định, threads, max concurrent, autostart, port REST API (15151), lưu nguyên tử Atomic JSON. | ✅ Completed |
| `SystemTray.kt` | Tauri System Tray API | Menu tray: Mở app, Pause All, Resume All, Thoát. | ⏳ Pending |

---

## 5. Browser Extension (`extension/` — Được Bảo Vệ & Quản Lý Tập Trung)

> 🛡️ **QUY TẮC BẢO VỆ:** Thư mục `extension/` là mã nguồn chính thức cho Chrome Web Store / Firefox Addons, **KHÔNG ĐƯỢC XÓA** khi dọn dẹp mã nguồn.

| File trong `extension/` | Vai trò trong hệ thống | Kết nối với Desktop | Trạng thái |
| :--- | :--- | :--- | :---: |
| `manifest.json` | Manifest V3 cấu hình quyền `downloads`, `storage`, `notifications` | `host_permissions: ["http://localhost/*"]` | ✅ Hoàn chỉnh |
| `background.js` | Service worker bắt link tải qua `onDeterminingFilename` | Gọi API `http://localhost:15151/add` và `GET /` | ✅ Hoàn chỉnh |
| `content.js` & `overlay.css` | Bắt link click trên trang web và hiển thị overlay icon tải | Gửi message `DOWNLOAD_LINK` cho `background.js` | ✅ Hoàn chỉnh |
| `popup.html` & `popup.js` | Popup bật/tắt tính năng bắt link & kiểm tra trạng thái kết nối App | Hiển thị trạng thái kết nối cổng 15151 | ✅ Hoàn chỉnh |

---

## 6. Landing Page & Website (`landing/` — Được Bảo Vệ & Quản Lý Tập Trung)

> 🛡️ **QUY TẮC BẢO VỆ:** Thư mục `landing/` là website chính thức của sản phẩm (`flowspeed.link`), **KHÔNG ĐƯỢC XÓA** khi dọn dẹp mã nguồn.

| File trong `landing/` | Vai trò trong hệ thống | Nền tảng triển khai | Trạng thái |
| :--- | :--- | :--- | :---: |
| `index.html` | Trang chủ giới thiệu tính năng, link tải app và hướng dẫn cài đặt extension | Cloudflare Pages / Static CDN | ✅ Hoàn chỉnh |
| `privacy.html` | Chính sách bảo mật bắt buộc cho Chrome Web Store submission | Cloudflare Pages / Static CDN | ✅ Hoàn chỉnh |
| `wrangler.jsonc` | Cấu hình tự động deploy lên Cloudflare Pages/Workers | Cloudflare Workers CLI (`wrangler`) | ✅ Hoàn chỉnh |
| `css/` & `js/` & `assets/` | Stylesheet, animations, hình ảnh minh họa sản phẩm | Assets tĩnh | ✅ Hoàn chỉnh |
