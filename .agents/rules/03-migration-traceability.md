# Quy Chuẩn Chuyển Dịch, Quản Lý Monorepo & Truy Vết

## 🎯 Nguyên Tắc Cốt Lõi

1. **Không Bỏ Sót Tính Năng (Zero-Gap Parity):**
   - Mọi module được chuyển dịch từ Kotlin (`legacy/kotlin-compose`) sang Rust (`crates/flow_core`, `crates/flow_server`, `crates/flow_desktop`) phải đối chiếu 1:1 với logic gốc.
   - Khi port bất kỳ tính năng nào, luôn kiểm tra hàm/class tương ứng trên nhánh legacy:
     `git show legacy/kotlin-compose:<path-to-kotlin-file>`

2. **Cập Nhật Ma Trận Truy Vết (Traceability Matrix):**
   - Sau mỗi khi hoàn thành hoặc cập nhật một module/chức năng, Agent **BẮT BUỘC** cập nhật trạng thái trong:
     - [TRACEABILITY_MATRIX.md](file:///d:/repos/flowspeed.link/docs/migration/TRACEABILITY_MATRIX.md)
     - [PROGRESS.md](file:///d:/repos/flowspeed.link/docs/migration/PROGRESS.md)

3. **Bảo Vệ Các Thành Phần Tập Trung (Monorepo Assets Protection):**
   - **`extension/` (Browser Extension Manifest V3):** Là mã nguồn chính thức của Extension Chrome/Firefox, **TUYỆT ĐỐI KHÔNG ĐƯỢC XÓA HOẶC DỌN DẸP NHẦM**.
   - **`landing/` (Official Website):** Là website chính thức của sản phẩm (`flowspeed.link`), **TUYỆT ĐỐI KHÔNG ĐƯỢC XÓA HOẶC DỌN DẸP NHẦM**.

4. **Bảo Đảm Bằng Kiểm Thử (Test-Driven Verification):**
   - Không được đánh dấu một tính năng là `Completed` nếu chưa có bài test `#[test]` tương đương trong Rust kiểm chứng các trường hợp biên (edge cases).
