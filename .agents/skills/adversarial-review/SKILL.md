---
name: adversarial-review
description: >
  Use for critical, rigorous peer code review, pre-merge validation, spec compliance checks,
  and regression risk inspection before submitting changes.
  Triggers: "code review", "review PR", "kiểm tra code", "soi lỗi", "adversarial review",
  "inspect diff", "phản biện code". Source: superpowers (requesting-code-review + receiving-code-review).
---

# Adversarial Code Review SOP

> ⚡ **TỐI ƯU TOKEN (TOKEN EFFICIENCY)**:
> - Chỉ review trên `git diff` (những dòng thay đổi) và các unit test liên quan trực tiếp.
> - Báo cáo lỗi theo mức độ nghiêm trọng: `[CRITICAL]`, `[WARNING]`, `[NIT]`.

---

## 1. Tâm Thế Review Phản Biện (The Reviewer Mindset)

Mục tiêu không phải là "khen ngợi code chạy được", mà là **chủ động tìm ra cách code có thể bị lỗi, crash hoặc bảo mật kém**.

---

## 2. Checklist Rà Soát 5 Tiêu Chí

1. **Spec Compliance**:
   - Code có thực hiện ĐÚNG và ĐỦ những gì user yêu cầu không?
   - Có thêm các tính năng thừa thãi ngoài phạm vi không (YAGNI)?
2. **Regression & Collateral Damage**:
   - Có vô tình làm hỏng các test case cũ hoặc API endpoints liên quan không?
   - Các file xung quanh có bị format/sửa ngoài ý muốn không?
3. **Edge Cases & Error Handling**:
   - `null`, `undefined`, chuỗi rỗng, mảng rỗng có bị unhandled crash không?
   - Network timeout, DB error có được bọc trong try/catch hoặc error boundary không?
4. **Security & Secrets**:
   - Có hardcode API key, token, credential nào không?
   - Input người dùng có qua sanitize/validate trước khi xử lý không?
5. **Fresh Evidence Check**:
   - Tác giả PR/Agent có đính kèm output thực thi lệnh test xanh (Pass) không?
