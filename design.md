# Tài liệu Hệ thống Thiết kế Giao diện (Design System)
## Phong cách: Cyber-Industrial / Brutalist Tech (Dark Mode)

Tài liệu này phân tích chi tiết phong cách thiết kế giao diện phần mềm và web app từ mã nguồn của dự án **Flow** (thư mục [landing](file:///home/gone/NewVolume_200G/repos/flowspeed.link/landing)), giúp định hình bộ quy chuẩn thiết kế từ tổng quan đến chi tiết để xây dựng các web app tương tự.

---

## 1. Triết Lý Thiết Kế Chủ Đạo (Design Philosophy)

Phong cách của **Flow** được định hình bởi trường phái **Cyber-Industrial (Công nghiệp tương lai)** kết hợp với **Brutalist Tech (Công nghệ thô mộc)**:

- **Bố cục Phẳng & Sắc cạnh**: Hạn chế tối đa việc bo góc tròn (`border-radius` bằng `0` hoặc cực kỳ nhỏ đối với các mockup đặc thù). Các khối được chia tách bằng hệ thống lưới (grid) viền mỏng như bản vẽ kỹ thuật.
- **Tương phản cực hạn (High Contrast Dark Mode)**: Sử dụng các sắc độ tối sâu (gần đen) làm nền, kết hợp với các dải màu nhấn (Neon/Orange) để tạo hiệu ứng phát sáng đặc trưng của màn hình điều khiển (Dashboard).
- **Hơi hướng Kỹ thuật (Technical Aesthetic)**: Sử dụng font chữ Monospace (Font đơn cách) cho các thông số kỹ thuật, nhãn (label), trạng thái để mang lại cảm giác chuyên nghiệp, chính xác.
- **Chuyển động & Chiều sâu**: Không dùng đổ bóng mờ kiểu truyền thống. Chiều sâu được tạo nên bằng các quầng sáng động (Ambient Glow), hiệu ứng hạt liên kết (Particle Net) và hiệu ứng phát sáng theo con trỏ chuột (Cursor Glow).

---

## 2. Hệ Thống Màu Sắc (Color Palette)

Hệ thống màu sắc được lưu trữ thông qua các biến CSS tùy chỉnh (CSS Variables) tại [base.css](file:///home/gone/NewVolume_200G/repos/flowspeed.link/landing/css/base.css):

```css
:root {
  /* Màu nhấn chủ đạo (Neon Orange) */
  --orange: #E64A00;          /* Màu cam chính cho các hành động quan trọng */
  --orange-light: #FF6B35;    /* Màu cam sáng khi hover */
  --orange-dim: rgba(230, 74, 0, 0.12); /* Màu cam mờ cho nền phụ, badge */

  /* Phân tầng màu nền tối (Dark Background Hierarchy) */
  --bg: #080808;              /* Nền gốc của trang web (gần như đen tuyệt đối) */
  --bg-2: #111111;            /* Nền của card, section con, hoặc container chính */
  --bg-3: #1a1a1a;            /* Nền của toolbar, input, hoặc các panel nhỏ */
  --bg-4: #222222;            /* Nền sâu nhất (ví dụ: rãnh của thanh tiến trình) */

  /* Hệ thống đường viền (Borders & Dividers) */
  --border: rgba(255, 255, 255, 0.07);   /* Viền mảnh phân tách layout nhẹ */
  --border-2: rgba(255, 255, 255, 0.12); /* Viền của các nút bấm hoặc thẻ */

  /* Phân tầng văn bản */
  --text: #E5E7EB;            /* Chữ chính (xám sáng nhẹ, giảm mỏi mắt so với trắng tinh) */
  --text-muted: #6B7280;      /* Chữ mờ cho chú thích, mô tả phụ */
  --text-dim: #4B5563;        /* Chữ cực mờ cho thông tin thứ cấp */

  /* Màu trạng thái bổ trợ */
  --green: #22C55E;           /* Thành công, hoàn tất tải xuống */
}
```

---

## 3. Hệ Thống Typography

Hệ thống chữ kết hợp giữa sự mượt mà của sans-serif hiện đại và sự góc cạnh của monospace kỹ thuật:

### 3.1. Các Font Chữ Sử Dụng
- **Font chính (Body & Headers thông thường)**: `'Inter', system-ui, sans-serif` đem lại sự tinh tế và dễ đọc khi hiển thị văn bản dài.
- **Font kỹ thuật (Stats, Labels, Code, Buttons, Badges)**: `'JetBrains Mono', monospace` tạo điểm nhấn công nghệ, tối ưu hiển thị số liệu.

### 3.2. Quy tắc Phân cấp Typography
- **Tiêu đề lớn (Hero Title)**: 
  - Kích thước: `clamp(42px, 7vw, 80px)` (Co giãn linh hoạt theo viewport).
  - Độ dày: `font-weight: 900` hoặc `800` (cực dày).
  - Khoảng cách chữ: `letter-spacing: -3px` (rất sát để tăng tính Brutalist).
  - Chiều cao dòng: `line-height: 1.0` (tiết kiệm không gian đứng).
- **Tiêu đề phân khu (Section Title)**:
  - Kích thước: `clamp(28px, 4vw, 48px)`.
  - Độ dày: `font-weight: 800`.
  - Khoảng cách chữ: `letter-spacing: -1.5px`.
  - Chiều cao dòng: `line-height: 1.1`.
- **Nhãn phân khu (Section Label)**:
  - Sử dụng Font Mono.
  - Định dạng: Viết hoa toàn bộ (`text-transform: uppercase`), cỡ chữ nhỏ `11px`, màu cam `--orange`, khoảng cách chữ giãn rộng (`letter-spacing: 3px`).
- **Chữ Gradient (Gradient Text)**:
  ```css
  .gradient-text {
    background: linear-gradient(135deg, var(--orange) 0%, var(--orange-light) 50%, #FFB347 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }
  ```

---

## 4. Các Thành Phần Giao Diện Đặc Trưng (UI Components)

### 4.1. Nút Bấm (Buttons)
Nút bấm được thiết kế không bo góc và phản hồi tương tác nhạy bén:

- **Nút chính (Primary Button)**:
  - Cấu trúc: Nền cam `--orange`, chữ trắng, chữ in đậm `font-weight: 700`.
  - Hiệu ứng: Khi hover, đổi nền thành `--orange-light`, nhấc nhẹ lên (`transform: translateY(-1px)`), và đổ bóng tỏa màu cam mờ (`box-shadow: 0 8px 24px rgba(230,74,0,0.3)`).
  - Lớp phủ chéo: Sử dụng pseudo-element `::before` với gradient nghiêng nhẹ để tạo hiệu ứng óng ánh khi hover.
- **Nút viền (Ghost Button)**:
  - Cấu trúc: Viền mảnh `--border-2`, chữ xám `--text-muted`.
  - Hiệu ứng: Khi hover, đổi màu viền thành `--text-muted` và màu chữ thành `--text` (hoặc chuyển sang sắc cam tùy ngữ cảnh).

### 4.2. Khung Giả Lập Cửa Sổ Ứng Dụng (App Window Mockup)
Đây là đặc trưng quan trọng nhất để trình diễn giao diện phần mềm trực quan ngay trên web (Xem tại [hero.css](file:///home/gone/NewVolume_200G/repos/flowspeed.link/landing/css/hero.css)):

- **Khung cửa sổ (`.app-window`)**: Nền màu `--bg-2`, đường viền sắc nét `--border-2`. Có một lớp bóng đổ cực sâu `box-shadow: 0 40px 100px rgba(0,0,0,0.7)` để tách biệt khỏi nền tối.
- **Thành phần thanh tiêu đề (`.window-bar`)**:
  - Nền tối hơn `--bg-3`.
  - Phía bên trái là 3 nút điều khiển dạng tròn màu Mac (`#FF5F57`, `#FFBD2E`, `#28C840`).
  - Tiêu đề cửa sổ căn giữa bằng font Mono cỡ nhỏ.
- **Bảng dữ liệu danh sách tải xuống (`.dl-row`)**:
  - Tổ chức dạng lưới CSS Grid để căn thẳng hàng các cột: Tên file, Dung lượng, Trạng thái, Tốc độ.
  - Phân cách hàng bằng các đường viền mảnh `1px solid var(--border)`.
  - Thanh tiến trình tải xuống (`.dl-bar` và `.dl-fill`): Thanh dẫn màu tối, thanh chạy màu cam `--orange` chuyển sang màu xanh `--green` khi hoàn tất.

### 4.3. Các Thẻ Đặc Tính (Feature Cards)
- Bố cục lưới khít nhau, dùng chung màu viền `--border` để tạo hệ lưới liên kết chặt chẽ.
- Khi di chuột qua card (`.feature-card:hover`):
  - Thay đổi nhẹ nền từ `--bg` sang `--bg-2`.
  - Một thanh nhỏ màu cam ở trên đỉnh thẻ trượt dài ra (`transform: scaleX(1)`) từ trạng thái ẩn nhờ hiệu ứng chuyển cảnh `transition: transform 0.3s`.

### 4.4. Dải Chữ Chạy (Marquee)
- Tạo chuyển động kỹ thuật liên tục ở ranh giới các khối:
  ```css
  .marquee-wrap {
    overflow: hidden;
    border-top: 1px solid var(--border);
    border-bottom: 1px solid var(--border);
    background: var(--bg-2);
    padding: 14px 0;
  }
  .marquee-track {
    display: flex; gap: 32px; width: max-content;
    animation: marquee 30s linear infinite;
    font-family: var(--mono);
    color: var(--text-muted);
  }
  @keyframes marquee {
    from { transform: translateX(0); }
    to { transform: translateX(-50%); }
  }
  ```

---

## 5. Hiệu Ứng Hoạt Họa & Tương Tác Kỹ Thuật (Interactive Effects)

Để web app mới cho cảm giác cao cấp và hiện đại, hãy áp dụng các hiệu ứng động sau:

### 5.1. Con Trỏ Phát Sáng (Cursor Glow)
Tạo một vùng sáng cam dịu đi theo chuột của người dùng:
- **HTML**: `<div class="cursor-glow" id="cursorGlow"></div>` đặt ngay dưới thẻ `body`.
- **CSS**:
  ```css
  .cursor-glow {
    position: fixed;
    width: 500px; height: 500px;
    border-radius: 50%;
    background: radial-gradient(circle, rgba(230,74,0,0.05) 0%, transparent 70%);
    pointer-events: none; /* Không cản trở click chuột */
    z-index: 0;
    transform: translate(-50%, -50%);
    transition: opacity 0.3s;
  }
  ```
- **JS** (Xem tại [ui.js](file:///home/gone/NewVolume_200G/repos/flowspeed.link/landing/js/ui.js)):
  ```javascript
  const cursorGlow = document.getElementById('cursorGlow');
  document.addEventListener('mousemove', e => {
    if (cursorGlow) {
      cursorGlow.style.left = e.clientX + 'px';
      cursorGlow.style.top = e.clientY + 'px';
    }
  });
  ```

### 5.2. Mạng Lưới Hạt Chuyển Động (Particle Network Canvas)
Lớp nền hoạt động phía sau Hero Section vẽ các điểm hạt và tự động kết nối chúng bằng đường thẳng mờ khi chúng ở gần nhau (Xem mã nguồn chi tiết tại [canvas.js](file:///home/gone/NewVolume_200G/repos/flowspeed.link/landing/js/canvas.js)).
- Sử dụng thẻ `<canvas>` chiếm trọn vùng nền của Hero.
- Cập nhật tọa độ hạt liên tục thông qua `requestAnimationFrame`.
- Kết nối các hạt cách nhau dưới `120px` bằng nét vẽ siêu mảnh màu cam nhạt `rgba(230,74,0, 0.08)`.

### 5.3. Hiệu Ứng Xuất Hiện Cuộn Trang (Scroll Reveal)
Dùng `IntersectionObserver` để kích hoạt hiệu ứng mượt mà khi người dùng cuộn tới:
- **CSS**:
  ```css
  .reveal {
    opacity: 0;
    transform: translateY(24px);
    transition: opacity 0.6s ease, transform 0.6s ease;
  }
  .reveal.visible {
    opacity: 1;
    transform: translateY(0);
  }
  ```
- **JS** (Xem tại [animations.js](file:///home/gone/NewVolume_200G/repos/flowspeed.link/landing/js/animations.js)):
  ```javascript
  const revealObserver = new IntersectionObserver((entries) => {
    entries.forEach(entry => {
      if (entry.isIntersecting) {
        const delay = entry.target.dataset.delay || 0;
        setTimeout(() => entry.target.classList.add('visible'), parseInt(delay));
      }
    });
  }, { threshold: 0.1 });
  document.querySelectorAll('.reveal').forEach(el => revealObserver.observe(el));
  ```

---

## 6. Hướng Dẫn Áp Dụng Cho Web App Mới

Khi triển khai một web app mới dựa trên bộ khung thiết kế này, hãy thực hiện theo các bước sau:

1. **Khởi tạo tệp tin `index.css`**: Nạp toàn bộ các biến CSS trong mục 2 và thiết lập cấu hình reset mặc định (chú ý `box-sizing: border-box`, triệt tiêu margin mặc định, đặt nền `--bg` cho `body`).
2. **Cài đặt font chữ**: Thêm liên kết Google Fonts cho `Inter` và `JetBrains Mono` vào thẻ `<head>` của trang web.
3. **Tuân thủ quy chuẩn thiết kế nút**: Không bo góc các nút kêu gọi hành động (CTA), áp dụng đúng mã màu `--orange` và các chuyển dịch vị trí khi tương tác.
4. **Xây dựng bố cục bằng Grid & Flexbox**: Đảm bảo cấu trúc lưới thẳng hàng, rõ ràng, các khối ngăn cách bằng đường kẻ mờ thay vì đổ bóng bề mặt.
5. **Tận dụng hiệu ứng Cursor Glow & Scroll Reveal** để giao diện tối không bị đơn điệu mà luôn có cảm giác chuyển động sống động.
