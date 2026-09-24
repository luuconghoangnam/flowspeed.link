# 🚀 Kế Hoạch Chuyển Dịch Toàn Diện: Flow Speed Link sang Tauri v2 (Rust + Tailwind/Svelte/Vue)

---

## 📌 1. Tổng Quan Kiến Trúc Mới (Target Architecture)

```
┌────────────────────────────────────────────────────────────────────────┐
│                   FRONTEND (Tauri v2 Webview ~15-20MB)                 │
│         Svelte 5 / Vue 3 + TailwindCSS + Lucide Icons + Shadcn         │
│  [Home / Downloads] [Add URL Modal] [Queue Manager] [Settings] [Hash]  │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │ Tauri IPC (Commands & Events)
┌───────────────────────────────────▼────────────────────────────────────┐
│                        TAURI V2 RUNTIME (Rust)                         │
│  - System Tray & Native Notifications                                  │
│  - Single Instance Lock (single_instance crate)                        │
│  - Autostart on boot & Global Shortcuts                                │
│  - Window Management & Drag-and-Drop                                   │
└──────────────┬──────────────────────────────────────────┬──────────────┘
               │                                          │
┌──────────────▼──────────────────────────┐   ┌───────────▼──────────────┐
│       CORE ENGINE (flow_core crate)     │   │  INTEGRATION IPC (Axum)  │
│  - Tokio Async Runtime (Zero-copy I/O)  │   │  - Port 15151 HTTP REST  │
│  - Multi-part HTTP Range (Reqwest)      │   │  - Chrome/Firefox Ext.   │
│  - HLS M3U8 Downloader (m3u8-rs)        │   │  - POST /add             │
│  - Sparse File (windows-sys FFI)        │   │  - POST /headless        │
│  - Checksum (sha2, md5, sha1)           │   │  - GET /queues           │
│  - Queue Manager & Concurrency Actors   │   └──────────────────────────┘
│  - Atomic JSON Storage (serde_json)     │
└─────────────────────────────────────────┘
```

---

## 🔄 2. Bảng Ánh Xạ Logic & Thư Viện (Kotlin $\rightarrow$ Rust + Tauri v2)

### A. Phân Hệ Core Downloader & Mạng

| Tính năng / Logic | Kotlin (Hiện tại) | Rust Equivalents | Ghi chú kỹ thuật |
| :--- | :--- | :--- | :--- |
| **Async Runtime & Threading** | Kotlin Coroutines (`Dispatchers.IO`) | **`tokio`** (full features) | Không bị GC pause, quản lý pool I/O siêu nhẹ. |
| **HTTP Client & Range Download** | OkHttp 5 + Okio Buffer | **`reqwest`** + **`bytes`** | Hỗ trợ HTTP/1.1, HTTP/2, Connection Pooling, Rustls native TLS. |
| **Tải đa luồng theo dải byte** | `HttpPartDownloader.kt` | `tokio::spawn` + `Range: bytes=start-end` | Ghi song song vào file buffer qua `tokio::fs::File` + `tokio::io::AsyncSeekExt`. |
| **HLS Stream (.m3u8 & .ts segments)**| `HLSDownloadJob.kt` (`m3u8-parser`) | **`m3u8-rs`** crate | Parse playlist M3U8, tải song song audio/video chunks và ghép tự động. |
| **Cơ chế Retry & Exponential Backoff**| `HttpRetryPolicy.kt` | Custom async backoff loop | `min(initial * 2^attempt, max_delay)` với jitter. |
| **Cấp phát file thưa (Sparse File)** | `SparseFile.kt` (Windows FFI) | **`windows-sys`** (`FSCTL_SET_SPARSE`) | Hỗ trợ cấp phát file dung lượng lớn tức thì trên NTFS mà không tốn thời gian zero-fill. |
| **Tính toán mã băm (Checksum)** | Okio Hashing (`ChecksumUtil.kt`) | **`sha2`**, **`sha1`**, **`md5`** crates | Tính toán streaming hash song song trong quá trình ghi đĩa, zero RAM overhead. |

---

### B. Phân Hệ Quản Lý Hàng Đợi (Queue Manager) & Lưu Trữ (Storage)

| Tính năng / Logic | Kotlin (Hiện tại) | Rust Equivalents | Ghi chú kỹ thuật |
| :--- | :--- | :--- | :--- |
| **Quản lý hàng đợi (Queue Manager)** | `QueueManager.kt`, `DownloadQueue.kt` | Actor pattern với **`tokio::sync::mpsc`** + `Arc<RwLock<QueueState>>` | Đảm bảo an toàn luồng tuyệt đối (Data Race Free), quản lý `max_concurrent` chính xác. |
| **Lập lịch tải (Scheduler)** | Auto start/stop time | **`tokio-cron-scheduler`** hoặc timer | Tự động kích hoạt/tạm dừng hàng đợi theo khung giờ cài đặt. |
| **Lưu trạng thái an toàn (Persistence)**| `TransactionalFileSaver.kt` (ghi .tmp rồi rename) | **`tempfile`** + `std::fs::rename` + **`serde_json`** | Ghi file cấu hình và tiến trình download nguyên tử (atomic), chống hỏng file khi mất điện đột ngột. |

---

### C. Phân Hệ Trình Duyệt Tích Hợp (Browser Extension Integration)

| Tính năng / Logic | Kotlin (Hiện tại) | Rust Equivalents | Ghi chú kỹ thuật |
| :--- | :--- | :--- | :--- |
| **Local HTTP Server** | `MyHttp4KServer.kt` (Port 15151) | **`axum`** + **`tower-http`** (CORS) | Chạy local server siêu nhẹ (ăn <2MB RAM), tiếp nhận request từ extension Chrome/Firefox. |
| **API Endpoints** | `/add`, `/queues`, `/start-headless-download` | Khớp chuẩn 100% `REST-API.yml` | Extension hiện tại dùng được ngay, không cần sửa đổi 1 dòng code extension nào. |
| **Khóa đơn phiên (Single Instance)** | `SingleInstanceServer.kt` (app.lock) | **`single_instance`** crate | Ngăn chặn mở trùng lặp app; chuyển tiếp URL từ trình duyệt sang cửa sổ đang chạy. |

---

### D. Phân Hệ Giao Diện Người Dùng (Desktop GUI)

| Màn hình & Chức năng | Kotlin Compose Desktop | Tauri v2 + Tailwind / Svelte / Vue |
| :--- | :--- | :--- |
| **Trang chủ (Home Page)** | Danh sách download, bảng tiến trình, filter | Bảng danh sách tải file mượt mà, virtualized list (chạy mượt dù có 10.000 link). |
| **Hộp thoại thêm link (Add Modal)** | Bóc tách URL, chọn folder, chọn Queue, bóc filename | Popup kính mờ (glassmorphic), tự bắt clipboard URL khi bấm phím tắt. |
| **Thước đo tốc độ (Speed Graph)** | Canvas custom draw | SVG Canvas / Chart.js / Recharts siêu đẹp, mượt 60 FPS. |
| **System Tray & Thu nhỏ** | AWT SystemTray | Tauri v2 System Tray native (Menu chuột phải, icon trạng thái, ẩn về tray khi đóng). |
| **Thông báo hệ thống** | Custom Notification / Toast | Tauri v2 `plugin-notification` (Native Windows 10/11 Notification banner). |
| **Khởi động cùng máy (Autostart)** | Registry script | Tauri v2 `plugin-autostart`. |

---

## 🗺️ 3. Lộ Trình Triển Khai Chi Tiết (4 Giai Đoạn)

### 🟢 Giai đoạn 1: Xây Dựng Core Engine Bằng Rust (`crates/flow_core`)
1. Cấu hình Cargo Workspace.
2. Xây dựng modules:
   - `downloader::range`: Range header, Multi-part task executor, Dynamic split.
   - `downloader::hls`: M3U8 Master & Media playlist parsing, media segment streaming.
   - `storage::sparse`: Cấp phát sparse file qua `windows-sys` FFI.
   - `storage::atomic`: Ghi file JSON transactional atomic.
   - `checksum`: Streaming SHA256/SHA1/MD5 engine.
   - `queue::manager`: Actor model điều phối các tác vụ tải.
3. Chuyển đổi Unit Tests từ Kotlin sang Rust `#[test]`.

### 🟡 Giai đoạn 2: Axum Extension Server & Single Instance (`crates/flow_server`)
1. Implement REST API port 15151 (`POST /add`, `GET /queues`, `POST /start-headless-download`).
2. Single-instance Mutex & IPC forwarding.

### 🟠 Giai đoạn 3: Tauri v2 Desktop GUI (`crates/flow_desktop`)
1. Scaffolding Tauri v2 + TailwindCSS + Svelte / Vue.
2. Tích hợp System Tray, Autostart, Dialog, Notification plugins.
3. Xây dựng giao diện Dark/Light mode tối giản, hiện đại.

### 🔴 Giai đoạn 4: Đóng Gói & Tối Ưu Hóa (Packaging & Release)
1. Tối ưu release profile (LTO, strip, codegen-units=1).
2. Tạo installer `.exe` / `.msi` qua Tauri CLI.
