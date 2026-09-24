---
name: diagram-designer
description: >
  Vẽ sơ đồ kiến trúc, workflow, sequence, ERD, C4, data-flow trực tiếp dưới dạng
  Mermaid/Graphviz/PlantUML hoặc HTML+SVG động chuẩn editorial. Kích hoạt khi:
  "vẽ diagram", "sơ đồ kiến trúc", "architecture diagram", "sequence flow",
  "ERD", "C4", "flowchart", "vẽ luồng dữ liệu", "không dùng mermaid lỗi thời".
  Source: agentic-awesome-skills/diagram-generator + c4-architecture + cathrynlavery/diagram-design + tt-a1i/archify.
---

# Diagram Designer

> "Vẽ sơ đồ bằng code chuẩn xác hơn drag-and-drop. Agent đọc code, không đọc ảnh."

---

## 1. Chọn Format Output Phù Hợp

Trước khi vẽ, xác định 1 trong 4 mode:

| Mode | Khi nào dùng | Format output |
|------|-------------|---------------|
| **Text Diagram** | Docs, Markdown, review dễ chỉnh sửa | Mermaid / Graphviz DOT / PlantUML trong code block |
| **C4 Architecture** | Tài liệu hoá codebase đa tầng (Code→Component→Container→Context) | Mermaid + `.md` files trong `C4-Documentation/` |
| **HTML+SVG Động** | Trình bày, báo cáo đẹp, animated flow | Single-file HTML với embedded SVG và CSS animation |
| **Editorial Diagram** | Chuẩn agency, không mermaid slop | SVG hoặc HTML với custom style, typography sạch |

---

## 2. Bảng Quyết Định Ngôn Ngữ Diagram

Dùng **Mermaid** trừ khi có lý do rõ ràng để dùng ngôn ngữ khác:

| Cần vẽ | Ưu tiên | Ghi chú |
|--------|---------|---------|
| Process flow, decision tree, swimlane | `Mermaid flowchart TD/LR` | Dễ embed vào Markdown |
| Sequence tương tác hệ thống | `Mermaid sequenceDiagram` | PlantUML nếu cần formal UML |
| State machine, lifecycle | `Mermaid stateDiagram-v2` | |
| Database schema, ERD | `Mermaid erDiagram` | |
| Class/interface model | `Mermaid classDiagram` | |
| Git history | `Mermaid gitGraph` | |
| Architecture với nhiều cluster | `Graphviz DOT` | Layout tốt hơn cho graph dày |
| UML formal | `PlantUML` | Wrap `@startuml` / `@enduml` |
| Custom visual khi text không đủ | `SVG` | Kiểm soát chính xác layout |
| Animated flow trình bày đẹp | `HTML + CSS animation` | Single self-contained file |

---

## 3. Quy Tắc Mermaid (Tránh Lỗi Phổ Biến)

```
✅ flowchart TD  — KHÔNG dùng graph TD (deprecated syntax)
✅ Node ID dùng ASCII: ingest_svc[Ingest Service]
✅ Quote labels có dấu câu: A["Label (with parens)"]
✅ Decision diamond: decide{Condition?}
✅ Edge label ngắn: -- success --> / -. async .->
✅ Subgraph cho swimlane/layer: subgraph Backend ...end
❌ KHÔNG dùng HTML tags trong label
❌ KHÔNG dùng ký tự đặc biệt trong ID node
❌ KHÔNG tạo sơ đồ vượt quá 20 node trong 1 diagram (tách ra)
```

---

## 4. Mode C4 Architecture — Quy Trình Đầy Đủ

Dùng khi: *"tài liệu hoá codebase"*, *"C4 architecture"*, *"giải thích toàn bộ hệ thống"*

### Giai đoạn 1 — Code Level (Bottom-Up)
- Duyệt tất cả subdirectory từ sâu nhất lên
- Tạo `C4-Documentation/c4-code-<dir>.md` cho mỗi thư mục
- Ghi đầy đủ: hàm, class, dependencies, signature

### Giai đoạn 2 — Component Level
- Nhóm code-level docs theo domain/boundary
- Tạo `C4-Documentation/c4-component-<name>.md`
- Tạo Mermaid diagram quan hệ giữa components

### Giai đoạn 3 — Container Level
- Map components → deployment units (Docker, K8s, Lambda)
- Tạo `C4-Documentation/c4-container.md`
- Tạo OpenAPI spec cho mỗi container API

### Giai đoạn 4 — Context Level
- Xác định personas (human + programmatic)
- Tạo `C4-Documentation/c4-context.md`
- Vẽ C4Context Mermaid diagram: System + Users + External Systems

**Cấu trúc output:**
```
C4-Documentation/
├── c4-code-*.md       # 1 file / subdirectory
├── c4-component-*.md  # 1 file / component
├── c4-component.md    # Master index + relationship diagram
├── c4-container.md    # Container map + API docs
├── c4-context.md      # Context diagram + user journeys
└── apis/
    └── <container>-api.yaml
```

---

## 5. Mode HTML+SVG Động (Editorial / Archify-Style)

Dùng khi cần output đẹp cho trình bày, không phải Markdown documentation.

### Yêu cầu bắt buộc cho single-file HTML diagram:
- **Self-contained**: Zero external dependencies, không CDN
- **Dark/Light mode**: `prefers-color-scheme` CSS media query
- **Animated flow**: CSS `stroke-dashoffset` animation cho arrows
- **Typography**: System font stack sạch, không dùng Google Fonts (tránh phụ thuộc network)
- **Export hint**: Ghi chú `Ctrl+S` để lưu, `Ctrl+P` để in/PDF
- **Mobile-safe**: `viewBox` SVG thay vì `width/height` cứng

### Template structure:
```html
<!DOCTYPE html>
<html lang="vi">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>[Diagram Title]</title>
  <style>
    /* Design tokens */
    :root {
      --bg: #0d0d0d; --surface: #1a1a1a; --border: #2a2a2a;
      --text: #e8e8e8; --accent: #6366f1; --flow: #10b981;
    }
    @media (prefers-color-scheme: light) {
      :root { --bg: #fafafa; --surface: #fff; --border: #e5e7eb;
              --text: #111; --accent: #4f46e5; --flow: #059669; }
    }
    /* Animated flow arrows */
    .flow-arrow { stroke-dasharray: 8 4; animation: flow 1.5s linear infinite; }
    @keyframes flow { to { stroke-dashoffset: -12; } }
  </style>
</head>
<body>
  <svg viewBox="0 0 800 600" role="img" aria-label="[Description]">
    <!-- diagram content -->
  </svg>
</body>
</html>
```

---

## 6. Quy Trình Thực Hiện

1. **Xác định intent**: Loại diagram, audience (dev/stakeholder/presentation)
2. **Chọn mode**: Text / C4 / HTML+SVG / Editorial
3. **Thu thập entities**: Normalize tên, label ngắn gọn
4. **Vẽ draft**: Ưu tiên rõ ràng hơn đầy đủ
5. **Self-validate**:
   - [ ] Syntax hợp lệ cho ngôn ngữ đã chọn
   - [ ] Label ngắn, không bị clip
   - [ ] Edges/flows phản ánh đúng input
   - [ ] Assumptions được ghi chú nếu input mơ hồ
6. **Output**: Diagram source + ghi chú giả định nếu cần

---

## 7. Giới Hạn

- Mermaid renderer trên một số nền tảng (Notion, GitHub) có thể khác nhau — test trước khi dùng `stateDiagram-v2` hoặc `sankey-beta`
- HTML+SVG phức tạp cần trình duyệt để render; không dùng trên terminal headless
- C4 full workflow tốn nhiều token — dùng cho codebase ≥ medium, không cần cho scripts nhỏ
