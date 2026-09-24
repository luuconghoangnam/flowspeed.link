---
name: systematic-debugging
description: Quy trình gỡ lỗi hệ thống bài bản theo nguyên lý First-Principles (Tái hiện -> Cô lập -> Đặt giả thuyết -> Sửa chính xác -> Phân tích tác động). Áp dụng khi gặp bug khó, crash, memory leak, race condition hoặc kết quả sai bất thường.
---

# Systematic Debugging Protocol — Quy trình Gỡ lỗi Hệ thống Bài bản

Khi gặp lỗi runtime, bug khó, hoặc hành vi bất thường, **KHÔNG BAO GIỜ đoán mò, thử nghiệm lung tung hoặc che giấu sự cố bằng try-catch rỗng**. Hãy tuân thủ nghiêm ngặt quy trình 5 bước sau:

---

## 1. Nguyên tắc cốt lõi (Core Directives)
1. **Không sửa code khi chưa đọc log**: Luôn đọc full error log, stack trace và dòng lệnh gây lỗi trước tiên.
2. **Không sửa triệu chứng (Symptom Masking)**: Không swallow exception, không trả về dummy fallback, không comment out assertion hỏng.
3. **Cô lập thay đổi (Surgical Fix)**: Mỗi lần sửa lỗi chỉ can thiệp vào nguyên nhân gốc rễ, giữ phạm vi thay đổi nhỏ nhất có thể.

---

## 2. Quy trình 5 bước gỡ lỗi (5-Step Debugging Workflow)

### Bước 1: Tái hiện lỗi tối thiểu (Minimal Reproduction)
- Xác định chính xác input, môi trường và chuỗi thao tác dẫn đến lỗi.
- Đơn giản hóa kịch bản tái hiện thành 1 script nhỏ hoặc 1 unit test ngắn nhất có thể.

### Bước 2: Đọc log & Lần ngược Call Stack (Trace & Inspect)
- Xác định điểm gãy chính xác: `File nào? Hàm nào? Dòng số mấy?`.
- Kiểm tra giá trị của các biến tại thời điểm sụp đổ (dùng debugger hoặc print log có ngữ cảnh).
- Kiểm tra lại giả định ban đầu: Biến có bị `null/undefined`? Thao tác bất đồng bộ (async/await) có bị bỏ sót? Khoảng trắng/encoding có sai lệch?

### Bước 3: Đặt giả thuyết & Xác nhận nguyên nhân gốc (Hypothesis Testing)
- Đặt ra câu hỏi: *"Tại sao giá trị X lại trở thành Y tại thời điểm Z?"*
- Tìm điểm vi phạm hợp đồng (Broken API Contract) hoặc sai lệch luồng dữ liệu (Dataflow Mutation).
- Chứng minh được lý do khiến lỗi xảy ra trước khi thực hiện viết code sửa.

### Bước 4: Sửa lỗi chính xác (Surgical Fix)
- Sửa trực tiếp vào logic bị lỗi.
- Giữ nguyên tất cả các docstrings, comment và logic không liên quan.
- Đảm bảo giữ đúng kiểu dữ liệu (Type Safety) và hợp đồng hàm (Signature Contract).

### Bước 5: Kiểm tra tác động phụ & Chống tái phát (Regression Guard)
- Chạy lại toàn bộ test suite để đảm bảo không làm gãy các module xung quanh.
- Viết bổ sung 1 unit test kiểm thử trường hợp vi phạm vừa sửa để phòng ngừa bug tái phát trong tương lai.

---

## 3. Ma trận chẩn đoán nhanh (Diagnostic Matrix)

| Biểu hiện lỗi | Nguồn nguyên nhân cần soi | Lệnh / Thao tác gỡ lỗi |
|---|---|---|
| `NullPointer` / `TypeError: cannot read prop of undefined` | Upstream API trả về rỗng hoặc chưa initialize state | Trace upstream data source, thêm null-check ở tầng tiếp nhận |
| `Race Condition` / Data không đồng bộ | Thao tác async chạy song song không theo thứ tự | Kiểm tra Mutex/Lock, Promise.all vs sequence, `await` bị thiếu |
| `Memory Leak` / RAM tăng dần | Unsubscribed Event Listener, Timer không clear, Connection Pool hở | Kiểm tra cleanup trong `useEffect`/`ngOnDestroy`/unmount, đếm active handles |
| `Infinite Loop` / CPU 100% | Điều kiện dừng vòng lặp sai, State mutation làm trigger re-render | Kiểm tra dependency array, exit criteria của recursion/loop |
| Test qua ở local nhưng tịt ở CI/CD | Môi trường lệch (Timezone, Env variables, Order-dependent test) | Kiểm tra Mock time, isolated state giữa các test case |
