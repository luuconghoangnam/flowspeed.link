---
name: database-mastery
description: Quy chuẩn tối ưu Cơ sở dữ liệu chuyên sâu (SQL Query optimization, Indexing, Transaction Isolation, Concurrency Control, Lock Handling, Redis Caching, Idempotency). Áp dụng cho PostgreSQL, MySQL, Redis, MongoDB.
---

# Database Mastery & Performance Standard — Chuẩn Tối ưu CSDL

> ⚡ **TỐI ƯU TOKEN (TOKEN OPTIMIZATION)**:
> Không tự động nạp các file trong `references/` vào bộ nhớ lúc ban đầu.
> Chỉ mở tài liệu chuyên sâu khi người dùng yêu cầu tối ưu DB cụ thể:
> - Quy chuẩn & Tối ưu PostgreSQL nâng cao: Tra cứu [postgresql-best-practices.md](references/postgresql-best-practices.md)

Thiết kế và truy vấn Cơ sở dữ liệu đúng cách quyết định 90% hiệu năng và độ tin cậy của hệ thống Backend.

---

## 1. Tối ưu SQL Query & Indexing (Query Performance)
- **Tránh N+1 Query**: Luôn dùng `JOIN` hoặc Batch Eager Loading (`include`, `preload`) thay vì gọi query trong vòng lặp `for`.
- **Chỉ SELECT các cột cần thiết**: Tuyệt đối không dùng `SELECT *` trong production; chỉ chọn đúng các cột được dùng.
- **Chiến lược Đánh Index**:
  - Tạo Index trên các cột nằm trong mệnh đề `WHERE`, `JOIN ON`, `ORDER BY`, `GROUP BY`.
  - Dùng **Composite Index** (Index kết hợp) theo nguyên tắc: Cột lọc bằng (=) xếp trước, cột lọc khoảng (<, >, LIKE) xếp sau.
  - Phân tích câu lệnh chậm bằng `EXPLAIN ANALYZE` để phát hiện `Seq Scan` (quét toàn bảng) và chuyển thành `Index Scan`.

---

## 2. Quản lý Concurrency & Locking (Đồng thời & Khóa dữ liệu)

### Idempotency Key (Tính bất biến của giao dịch)
- Đảm bảo các API quan trọng (Thanh toán, Trừ tiền, Đặt hàng) có tham số `idempotency_key` lưu vào Redis/DB với Unique Constraint. Tránh tình trạng người dùng bấm nút 2 lần dẫn đến bị trừ tiền trùng.

### Khóa dữ liệu (Locking Strategy)
- **Optimistic Locking (Khóa lạc quan)**: Thích hợp cho đọc nhiều, ghi ít. Dùng cột `version` hoặc `updated_at`.
  ```sql
  UPDATE accounts SET balance = balance - 100, version = version + 1 
  WHERE id = 42 AND version = 1;
  ```
- **Pessimistic Locking (Khóa bi quan)**: Dùng cho các giao dịch nhạy cảm (Ví tiền, Đặt vé ghế):
  ```sql
  SELECT * FROM accounts WHERE id = 42 FOR UPDATE;
  ```

---

## 3. Chiến lược Caching với Redis (Caching Best Practices)
- **Cache-Aside Pattern**: Read từ Redis -> Miss DB -> Write DB -> Populate Redis với TTL.
- **Tránh Cache Stampede (Thất thoát Cache đột ngột)**: Đặt ngẫu nhiên TTL (TTL jitter = base + random(0..60s)) để các cache key không bị hết hạn cùng 1 giây.
- **Invalidation chuẩn xác**: Mỗi khi lệnh `UPDATE`/`DELETE` thành công ở DB, bắt buộc phải xóa/cập nhật cache key tương ứng ngay lập tức.
