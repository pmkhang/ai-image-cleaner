# AI Image Cleaner (`ai-image-cleaner`)

Công cụ dòng lệnh (CLI) hiệu năng cao viết bằng Rust giúp loại bỏ hoàn toàn các siêu dữ liệu AI (C2PA, EXIF, XMP, IPTC, PNG text chunks) và làm suy yếu các watermark vô hình (Google SynthID, Stable Signature) khỏi hình ảnh do AI tạo ra (Midjourney, Stable Diffusion, ComfyUI, DALL-E) để các mạng xã hội (Facebook, Instagram, X, TikTok) không tự động phát hiện và gắn nhãn AI.

---

## Tính năng chính

- 🚀 **100% Loại bỏ Metadata & C2PA (Standard Mode):**
  - Xóa sạch toàn bộ C2PA / Content Credentials manifest (JUMBF boxes).
  - Xóa toàn bộ EXIF, XMP, IPTC.
  - Xóa toàn bộ thông số prompt và workflow lưu trong các chunk PNG (`tEXt`, `zTXt`, `iTXt`).
  - Tái tạo container ảnh nguyên bản, giữ nguyên 100% chất lượng pixel.
- 🔬 **Chế độ nâng cao (`--deep` / `--synthid`):**
  - Tác động vi mô (micro-perturbation) lên pixel để phá cấu trúc watermark ẩn tần số cao (như SynthID, Tree-Rings).
  - 3 mức cường độ điều chỉnh được: `--strength low | medium | high`.
  - Mắt thường không thể phân biệt được sự khác biệt so với ảnh gốc.
- ⚡ **Xử lý hàng loạt đa luồng (Multi-threaded Batching):**
  - Tận dụng tối đa mọi nhân CPU thông qua thư viện Rayon.
  - Quét đệ quy thư mục với cờ `--recursive`.
- 🛡️ **An toàn dữ liệu:**
  - Mặc định xuất ảnh sạch ra thư mục riêng (`./cleaned/` hoặc `--output <path>`) để không làm mất file gốc.
  - Cung cấp cờ `--in-place` nếu muốn ghi đè trực tiếp.
  - Cung cấp cờ `--dry-run` để kiểm tra các thẻ metadata AI mà không thay đổi file.

---

## Cài đặt & Biên dịch

Yêu cầu Rust (phiên bản 1.80+):

```bash
# Biên dịch bản tối ưu (Release binary)
cargo build --release

# Binary sẽ nằm tại: target/release/ai-image-cleaner
```

---

## Hướng dẫn sử dụng

### 1. Làm sạch một file ảnh đơn lẻ

```bash
# Lưu file sạch vào thư mục ./cleaned/
ai-image-cleaner my_ai_photo.png

# Chỉ định đường dẫn file đầu ra cụ thể
ai-image-cleaner my_ai_photo.png -o clean_photo.png

# Ghi đè trực tiếp lên file gốc
ai-image-cleaner my_ai_photo.png --in-place
```

### 2. Xử lý hàng loạt toàn bộ thư mục

```bash
# Quét toàn bộ ảnh trong thư mục và lưu vào ./output_folder/
ai-image-cleaner ./my_ai_images/ -o ./output_folder/

# Quét đệ quy cả các thư mục con
ai-image-cleaner ./my_ai_images/ --recursive -o ./output_folder/
```

### 3. Bật chế độ Deep (Phá watermark ẩn SynthID)

```bash
ai-image-cleaner my_ai_photo.png --deep --strength medium -o clean_photo.png
```

### 4. Kiểm tra trước (Dry-run mode)

Kiểm tra xem ảnh có chứa metadata AI, prompt hay C2PA không mà không ghi file:

```bash
ai-image-cleaner ./my_ai_images/ --dry-run --verbose
```

---

## Chạy kiểm thử

```bash
cargo test
```
