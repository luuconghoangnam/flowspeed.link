---
name: unit-testing-tdd
description: Nguyên tắc viết Unit Test & Integration Test chuyên sâu theo chu trình TDD (Red-Green-Refactor). Áp dụng cho Vitest, Jest, Pytest, Go testing, JUnit, PHPUnit để viết bộ test tin cậy, không rác, bao phủ toàn bộ edge cases.
---

# Unit Testing & TDD Standard — Chuẩn mực Kiểm thử Đơn vị

> ⚡ **TỐI ƯU TOKEN (TOKEN OPTIMIZATION)**:
> Không tự động nạp các file trong `references/` vào bộ nhớ lúc ban đầu.
> Chỉ mở tài liệu hướng dẫn cụ thể theo framework khi người dùng yêu cầu:
> - Hướng dẫn Vitest: Tra cứu [vitest-guide.md](references/vitest-guide.md)
> - Hướng dẫn Jest: Tra cứu [jest-guide.md](references/jest-guide.md)
> - Hướng dẫn Pytest: Tra cứu [pytest-guide.md](references/pytest-guide.md)
> - Hướng dẫn Cypress: Tra cứu [cypress-guide.md](references/cypress-guide.md)

Một bộ test tốt giúp phát triển nhanh hơn 10 lần và loại bỏ nỗi sợ refactor. Hãy áp dụng chuẩn mực kiểm thử chuyên nghiệp dưới đây:

---

## 1. Chu trình TDD (Red - Green - Refactor)
1. **RED**: Viết test case mô tả kỳ vọng TRƯỚC KHI viết code xử lý (xác nhận test FAIL).
2. **GREEN**: Viết lượng code TỐI THIỂU để test case vừa tạo PASS.
3. **REFACTOR**: Tối ưu cấu trúc code cho sạch đẹp mà test vẫn luôn PASS.

---

## 2. Cấu trúc 3A tiêu chuẩn cho Test Case (Given-When-Then)
Mọi test case phải tuân theo cấu trúc **Arrange - Act - Assert**:

```typescript
test('nên tính tổng tiền chính xác khi áp dụng mã giảm giá 10%', () => {
  // 1. Arrange (Chuẩn bị dữ liệu mẫu & mock dependencies)
  $cart = new Cart([{ price: 100, qty: 2 }]);
  $discountService = new DiscountService({ rate: 0.1 });

  // 2. Act (Thực thi hàm cần kiểm thử)
  $total = $discountService.calculateTotal($cart);

  // 3. Assert (Xác minh kết quả trả về & tương tác)
  expect($total).toBe(180);
});
```

---

## 3. Ma trận Bao phủ Edge Cases (Edge Case Checklist)
Khi viết unit test cho bất kỳ hàm nào, bắt buộc phải kiểm tra qua ma trận 5 nhóm edge cases:

1. **Boundary Values (Giá trị biên)**: `0`, `-1`, `MAX_INT`, chuỗi rỗng `""`, mảng rỗng `[]`.
2. **Null & Undefined**: Giá trị thiếu, null reference, thuộc tính không tồn tại.
3. **Invalid Formatting**: Email sai định dạng, số điện thoại chứa chữ, JSON hỏng.
4. **Concurrency & Timing**: Hai request đồng thời, network timeout, clock drift.
5. **Security & Authorization**: Gửi token hết hạn, gọi hàm không có quyền admin.

---

## 4. Quy tắc Mocking sạch (Clean Mocking Directives)
- **Không mock những thứ đang test**: Chỉ mock các hệ thống bên ngoài (External DB, Third-party HTTP API, Payment Gateway, File I/O).
- **Tránh Over-Mocking**: Đừng mock nguyên cả framework hay ORM nếu có thể dùng In-memory Database / SQLite / Test Containers.
- **Tái lập trạng thái Mock**: Rơ-le lại các mock sau mỗi test case (`afterEach(() => jest.clearAllMocks())`) để tránh lây nhiễm state giữa các test case với nhau.
