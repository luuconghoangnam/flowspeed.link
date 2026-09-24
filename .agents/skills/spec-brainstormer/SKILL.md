---
name: spec-brainstormer
description: >
  Use before creative or complex engineering work: exploring user intent, resolving ambiguities,
  classifying path (Spike, Bounded, Architectural), and drafting implementation specs with TDD criteria.
  Triggers: "brainstorm", "spec", "plan feature", "lên kế hoạch", "thiết kế tính năng",
  "phân tích yêu cầu", "làm rõ spec". Source: superpowers (brainstorming + writing-plans).
---

# Spec & Brainstorming SOP

> ⚡ **TỐI ƯU TOKEN (TOKEN EFFICIENCY)**:
> - Chỉ hỏi ĐÚNG 1-2 câu trọng tâm vào mục tiêu cốt lõi (intent), không hỏi lan man những điều đã có trong codebase.
> - Bản spec súc tích: chia theo Phase, mỗi task có tiêu chí Red/Green rõ ràng.

---

## 1. Phân Loại 3 Cấp Độ (The 3 Paths)

1. **Spike (Thử nghiệm nhanh)**: Yêu cầu chưa rõ tính khả thi công nghệ.
   - *Hành động*: Viết đoạn script mẫu nhỏ trong thư mục tạm/scratch để chứng minh giải pháp.
2. **Bounded (Tính năng giới hạn)**: Thêm 1 API, 1 component, hoặc 1 logic cụ thể.
   - *Hành động*: Làm rõ spec trực tiếp trong chat ➔ Bắt tay thực hiện.
3. **Architectural (Thay đổi hệ thống lớn)**: Đổi database, kiến trúc auth, microservice.
   - *Hành động*: Bắt buộc tạo bản `implementation_plan.md` đầy đủ với user review.

---

## 2. Hard Gate: Không Code khi Chưa Rõ Intent

- **Discover Intent**: Người dùng muốn đạt được điều gì cuối cùng? Ai là người dùng tính năng này?
- **Surface Assumptions**: Nêu rõ các giả định ngầm. Nếu có 2 cách hiểu ➔ Hỏi ngay, không tự đoán.
- **Spec Checklist**:
  - [ ] Input & Output mong đợi.
  - [ ] Trường hợp biên (Edge cases & Failure modes).
  - [ ] Lệnh kiểm chứng tự động (Unit test command).
