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
| `QueueManager.kt`<br>`DownloadQueue.kt` | `flow_core::queue::manager` | - Quản lý nhiều hàng đợi tải độc lập.<br>- Giới hạn số file tải đồng thời (`maxConcurrent`).<br>- Tự động kích hoạt task kế tiếp khi task hiện tại xong. | `test_queue_lifecycle_and_concurrency` | ✅ Completed |
| `TransactionalFileSaver.kt`<br>`DownloadListFileStorage.kt` | `flow_core::storage::atomic` | - Ghi nguyên tử qua file `.tmp` và rename `std::fs::rename`.<br>- JSON Serialization qua `serde_json`.<br>- Chống hỏng dữ liệu khi crash/mất điện. | `test_atomic_save_and_load`<br>`test_queue_persistence_save_and_load` | ✅ Completed |
| Schedule times (Auto-start/stop) | `flow_core::queue::manager` | - Lập lịch tự động bật/tắt queue theo giờ cấu hình (`auto_start_time`, `auto_stop_time`). | `test_queue_lifecycle_and_concurrency` | ✅ Completed |

---

## 3. Browser Integration & IPC (`integration:server` $\rightarrow$ `crates/flow_server`)

| API / Endpoint (`REST-API.yml`) | Handler Rust (Axum) | Payload & Contract | Tương thích Extension | Trạng thái |
| :--- | :--- | :--- | :--- | :---: |
| `GET /` | `routes::handle_health_check` | Health check endpoint kiểm tra app đang chạy | ✅ 100% `background.js` | ✅ Completed |
| `POST /add` | `routes::handle_add_downloads` | Request: `{ items: [...], options: {...} }` hoặc `[ { link, ... } ]` | ✅ 100% `background.js` | ✅ Completed |
| `GET /queues` | `routes::handle_get_queues` | Response: `[ { id: 1, name: "Mặc định" } ]` | ✅ 100% Chrome/Firefox Ext | ✅ Completed |
| `POST /start-headless-download` | `routes::handle_headless_download`| Request: `{ downloadSource, folder, name, queueId }` | ✅ 100% Chrome/Firefox Ext | ✅ Completed |
| `SingleInstanceServer.kt` | `flow_server::main` (`SingleInstance`) | - Mutex Lock chống mở 2 app cùng lúc. | ✅ `SingleInstance` Lock | ✅ Completed |

---

## 4. Pure Native Rust Desktop GUI (`desktop:app` $\rightarrow$ `crates/flow_desktop_slint`)

| Màn hình / Component Kotlin (Compose) | Component Slint (Pure Native Rust) | Chức năng chi tiết | Trạng thái |
| :--- | :--- | :--- | :---: |
| `HomeScreen.kt` | `ui/appwindow.slint`<br>`ui/components/task_row.slint` | Danh sách tải file, tiến độ %, tốc độ MB/s, dung lượng, trạng thái, nút tạm dừng/tiếp tục/mở thư mục. | ✅ Completed |
| `Toolbar / SpeedMeter` | `ui/components/toolbar.slint`<br>`ui/components/speed_chart.slint` | Thao tác nhanh (Thêm link, Tải loạt, Dừng/Tiếp tục tất cả, Checksum, Toggle Dark/Light) & Biểu đồ sóng tốc độ Live. | ✅ Completed |
| `AddDownloadDialog.kt` | `ui/dialogs/add_modal.slint` | Dán URL tải, chọn thư mục lưu tệp, tùy chỉnh số luồng kết nối. | ✅ Completed |
| `BatchDownloadDialog.kt` | `ui/dialogs/batch_modal.slint` | Nhập danh sách nhiều URL + Tự động sinh link hàng loạt theo mẫu `[01-20]`, `[a-z]`. | ✅ Completed |
| `ChecksumDialog.kt` | `ui/dialogs/checksum_modal.slint` | Kiểm tra và so khớp mã băm Streaming SHA-256 / SHA-1 / MD5. | ✅ Completed |
| `SettingsScreen.kt` | `ui/dialogs/settings_modal.slint` | Cài đặt hệ thống 3 Tab: Tải về (Thư mục, Luồng, Đồng thời), Băng thông (Giới hạn tốc độ, Cổng server), Tiện ích (Clipboard, Khay hệ thống, Âm thanh). | ✅ Completed |
| `Sidebar / Categories` | `ui/components/sidebar.slint` | Phân loại tệp tin theo danh mục (Tất cả, Video, Âm nhạc, Nén, Tài liệu, Phần mềm) kèm bộ đếm số lượng. | ✅ Completed |
| `SingleInstanceServer.kt` | `crates/flow_desktop_slint/src/main.rs` | Nhúng Axum Extension Server (cổng 15151) chạy nền song song với vòng lặp sự kiện Slint. | ✅ Completed |

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
