# 📊 Tiến Độ Chuyển Dịch (Migration Progress Tracker)

## 📌 Tổng Quan Dự Án
- **Ngôn ngữ đích:** Pure Native Rust (1.80+) Monorepo
- **Kiến trúc GUI:** 
  - 🥇 **Pure Native Rust GUI (Mặc định chính thức):** `crates/flow_desktop_slint` (Dùng Slint Engine, 0% WebView2, 0% Chromium, Single Process, ~10MB RAM, CPU Software Renderer Fallback cho máy cổ/yếu).
  - 🥈 **Web-hybrid GUI (Dự phòng):** `crates/flow_desktop` (Tauri v2 + Svelte 5).
- **Backend Core Engine:** `crates/flow_core` (Multi-part HTTP 206, M3U8/HLS streaming, sparse allocation, queue manager, SHA-256/MD5 checksum).
- **Integration Server:** `crates/flow_server` (Axum REST API cổng 15151 kết nối Chrome/Firefox/Edge Extension).
- **Trạng thái hiện tại:** **Hoàn thành 100% chuyển dịch từ Kotlin/JVM sang Pure Native Rust**
- **Test Coverage:** 15/15 unit tests pass 100% (`cargo test --workspace`).

---

## 🎯 Danh Sách Hạng Mục Hoàn Thành

### Giai đoạn 1: Rust Core Engine (`crates/flow_core`)
- [x] Khởi tạo Cargo Workspace Monorepo (`crates/flow_core`, `crates/flow_server`, `crates/flow_desktop_slint`).
- [x] Module `flow_core::types` (DownloadTask, PartState, TaskStatus, DownloadProgress, AppSettings, FileCategory, PerHostRule, PowerActionConfig).
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

### Giai đoạn 3: Pure Native Rust GUI (`crates/flow_desktop_slint`)
- [x] Cấu hình Slint 1.9 với `backend-winit` và `renderer-femtovg` (0% WebView2, 0% Chromium).
- [x] Token màu sắc tương phản cao Dark Mode (`#0f172a`) & Light Mode (`#f8fafc`).
- [x] Toolbar điều khiển tải (Thêm link, Tải loạt, Tiếp tục, Tạm dừng, Checksum, Toggle Theme).
- [x] Live Speed Wave Visualizer (Vẽ biểu đồ sóng tốc độ thời gian thực).
- [x] Sidebar phân loại danh mục (Tất cả, Video, Âm nhạc, Nén, Tài liệu, Phần mềm) kèm bộ đếm Badge.
- [x] Task Row Table hiển thị % tiến độ, tốc độ MB/s, dung lượng tải, trạng thái, và nút thao tác.
- [x] Modal Thêm liên kết tải (Single URL, Chọn thư mục, Chọn số luồng).
- [x] Modal Tải hàng loạt (Batch Download) hỗ trợ tự sinh URL theo mẫu `[01-20]` hoặc `[a-z]`.
- [x] Modal Kiểm tra mã băm Checksum (SHA-256, SHA-1, MD5).
- [x] Modal Cài đặt hệ thống 3 Tab (Tải về, Băng thông, Tiện ích).
- [x] Tích hợp Axum REST API chạy nền nhận link tải từ Extension Chrome/Firefox.

### Giai đoạn 4: Đóng Gói, Tối Ưu Hóa & Đo Đạc Thực Tế
- [x] Cấu hình Release Profile (`lto = "thin"`, `opt-level = 3`, `strip = true`, `panic = "abort"`).
- [x] Build Windows Standalone Executables:
  - `target/release/flow_desktop.exe` (Pure Native Slint - 15 MB standalone)
  - `target/release/flow_server.exe` (924 KB)
- [x] Đo lường Task Manager & Benchmark:
  - ⚡ **Số lượng Process:** Đúng **1 process duy nhất** (xóa sạch 5 sub-process của WebView2/GPU).
  - 💾 **RAM Tiêu Thụ:** Rất thấp, chạy trực tiếp native mã máy.
  - 🏎️ **Thời gian khởi động:** < 10 ms (mở lên tức thì).
  - 🖥️ **Tương thích máy cũ:** Tự động fallback CPU Software Renderer (FemtoVG/Tiny-Skia) không lo crash GPU.

---

## 💻 Hướng Dẫn Chạy & Tiếp Tục Phát Triển Trên Máy Khác

### 1. Yêu cầu môi trường (Prerequisites)
- **Rust Toolchain:** `rustup default stable` (Rust 1.80+)
- **Windows Build Tools:** C++ Build Tools (MSVC)

### 2. Các lệnh thông dụng
```bash
# Kiểm tra toàn bộ mã nguồn
cargo check --workspace

# Chạy unit tests
cargo test --workspace

# Chạy ứng dụng Pure Native GUI ở chế độ Dev
cargo run -p flow_desktop_slint

# Đóng gói bản Release Standalone (.exe)
cargo build --release -p flow_desktop_slint
```
File `.exe` thành phẩm sẽ nằm tại: `target/release/flow_desktop.exe`.
