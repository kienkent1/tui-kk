# TUI-KK 🐳

**`tui-kk`** là ứng dụng **Terminal UI (TUI)** hiệu năng cao được viết bằng **Rust** (`ratatui`, `bollard`, `tokio`, `crossterm`) giúp quản lý và giám sát container trực tiếp trên terminal.

Dự án giao tiếp trực tiếp với **Docker Engine API** thông qua Unix Socket, Named Pipe hoặc HTTP/TCP, do đó hoạt động mượt mà với **Docker Engine CLI**, **Podman**, **Colima**, **OrbStack**, **Rancher Desktop**, cũng như **Docker Desktop**.

---

## 1. Dự án làm gì? (Tính năng chính)

Giao diện gồm **Sidebar điều hướng** bên trái (hỗ trợ cả phím tắt lẫn click chuột) và 4 phân hệ chính:

1. **Dashboard (`1. Dashboard`)**
   - Thẻ tổng quan Engine (phiên bản, OS/Arch, số CPU, tổng RAM, trạng thái Containers & Images).
   - Biểu đồ thời gian thực (**Gauge** + **Braille Line Chart**) theo dõi **CPU (%)** và **RAM Usage** của toàn bộ container đang chạy.
   - Biểu đồ cột (**BarChart**) phân bổ dung lượng **Storage** (`Images`, `Containers`, `Volumes`) và bảng thống kê tài nguyên chi tiết (`CPU %`, `MEM USAGE / LIMIT`, `MEM %`, `STORAGE RW / ROOTFS`) cho từng container.
2. **Containers (`2. Containers`)**
   - Tìm kiếm (chế độ `local` hoặc `remote`), lọc và sắp xếp theo `Name`, `Image`, `State`, `Created`.
   - Thao tác vòng đời trực tiếp bằng phím tắt hoặc nút bấm chuột trên cột **Action**: `Start (▶)`, `Stop (■)`, `Restart`, `Remove (🗑)`.
   - Màn hình **Container Detail** gồm 4 tab: `1. Overview`, `2. Network & Mounts`, `3. Env`, `4. Raw JSON`.
3. **Images (`3. Images`)**
   - Danh sách image với tìm kiếm, sắp xếp (`RepoTag`, `Id`, `Size`, `Created`, `Containers`), xóa image (`Remove`) và dọn dẹp dangling images (`Prune`).
   - **Run Image (Tạo & chạy container mới kiểu Docker Desktop)**:
     - Cấu hình **Container Name**, **Port Mappings** (`host:container[/proto]`), **Volume Mounts** (`host:container[:ro]`).
     - **Parse file `.env`**: nhập đường dẫn file `.env` để tự động đọc và nạp hàng loạt biến môi trường (hỗ trợ comment `#`, tiền tố `export `, chuỗi đơn/kép `'...'` / `"..."`, và tự động gộp/ghi đè key trùng).
     - **Thêm/Sửa/Xóa biến môi trường trực tiếp**: nhập từng cặp `KEY` / `VALUE`, paste nhiều dòng `.env`, hoặc nạp các biến `Env` mặc định từ Image (`Import Image Env`).
   - Màn hình **Image Detail** gồm 4 tab: `1. Overview`, `2. Layers`, `3. Config & Env`, `4. Raw JSON`.
4. **Logs (`4. Logs`)**
   - Quản lý các file log được ghi tại thư mục `log_dir` do người dùng cấu hình.
   - Hiển thị dung lượng, thời gian cập nhật, thống kê số dòng `ERROR` / `WARN` của từng file log.
   - Xem chi tiết nội dung log theo 4 tab lọc mức độ (`1: All`, `2: Errors Only`, `3: Warnings + Errors`, `4: Raw Lines`).
   - Hỗ trợ **làm trống (`Clear`)**, **xóa từng file (`Delete`)**, hoặc **xóa toàn bộ file log (`Delete All`)**.

---

## 2. Hướng dẫn tải và cài đặt

### Yêu cầu hệ thống
- **Rust toolchain**: `>= 1.88.0` (Rust Edition 2024).
- **Container Engine**: Docker Engine (`dockerd`), Podman (`podman.socket`), Colima, OrbStack, Rancher Desktop, hoặc Docker Desktop đang chạy.
- *(Tùy chọn)* **Nerd Font** (ví dụ: *JetBrainsMono Nerd Font*) nếu sử dụng `icon_set = "nerd"`.

### Tải mã nguồn & Build

```bash
# 1. Clone repository
git clone <repository-url>
cd tui-kk

# 2. Build bản debug
cargo build

# 3. Build bản release (tối ưu hiệu năng)
cargo build --release

# 4. (Tùy chọn) Cài đặt binary vào ~/.cargo/bin
cargo install --path .
```

---

## 3. Lệnh chạy & Phím tắt

### Các lệnh chạy ứng dụng

```bash
# Chạy trực tiếp bằng Cargo
cargo run

# Chạy bản release
cargo run --release

# Tuỳ chỉnh tốc độ khung hình (FPS) và tần số cập nhật (Tick rate Hz) (từ 1.0 đến 240.0)
cargo run -- --frame-rate 60 --tick-rate 4

# Kiểm tra hiển thị icon (ASCII / Nerd Font) trên terminal hiện tại
cargo run -- --icon-preview

# Xem trợ giúp CLI
cargo run -- --help
```

### Phím tắt cơ bản

| Phím | Chức năng | Phạm vi |
| :--- | :--- | :--- |
| `Tab` / `Shift+Tab` | Chuyển qua lại giữa các trang trên Sidebar | Toàn cục (Global) |
| `j` / `↓`, `k` / `↑` | Di chuyển con trỏ lên / xuống hoặc cuộn trang | Toàn cục |
| `Enter` | Xem chi tiết mục đang chọn / Xác nhận | Toàn cục |
| `Esc` | Quay lại danh sách / Thoát chế độ tìm kiếm | Toàn cục |
| `/` | Bắt đầu tìm kiếm (Search) | Containers, Images, Logs |
| `s` / `o` | Đổi tiêu chí sắp xếp (`CycleSort`) / Đảo chiều (`ToggleOrder`) | Containers, Images, Logs |
| `F5` | Làm mới dữ liệu (`Refresh`) | Toàn cục |
| `s` / `Ctrl+R` / `d` | Start-Stop / Restart / Xóa container | Containers |
| `r` | Mở màn hình **Run Image** (tạo container & nạp `.env`) | Images |
| `p` / `d` | Prune dangling images / Xóa image | Images |
| `c` / `d` / `D` | Làm trống file log (`Clear`) / Xóa file log / Xóa tất cả log | Logs |
| `l` | Chuyển nhanh tới trang **Logs** | Toàn cục |
| `Shift+M` | Bật / tắt chế độ bắt sự kiện chuột (`MouseCapture`) | Toàn cục |
| `q` / `Ctrl+C` | Thoát ứng dụng | Toàn cục |

---

## 4. Hướng dẫn chạy Test

Toàn bộ unit tests được đặt tập trung tại thư mục [`src/tests/`](src/tests/):

```bash
# Chạy toàn bộ test suite
cargo test

# Chạy test kèm hiển thị log/stdout
cargo test -- --nocapture

# Chạy riêng từng module test
cargo test dashboard_test
cargo test containers_test
cargo test images_test
cargo test logs_test
cargo test click_tracker_test
```

---

## 5. Hướng dẫn cấu hình khi dùng Engine CLI khác (Không dùng Docker Desktop)

`tui-kk` đọc cấu hình từ file:
1. **Cục bộ (Ưu tiên)**: `.config/config.toml` (ngay trong thư mục làm việc của dự án).
2. **Toàn cục theo hệ điều hành**:
   - **Linux / macOS**: `~/.config/tuikk/config.toml`
   - **Windows**: `C:\Users\<User>\AppData\Roaming\tuikk\config.toml`

### Cấu trúc file `config.toml`

```toml
theme = "dark"               # "dark", "light", hoặc tên theme trong theme.json5
log_level = "warn"           # "trace", "debug", "info", "warn", "error"
query_mode = "local"         # "local" (lọc trên RAM) hoặc "remote" (lọc qua API)
log_dir = "./logs"           # Thư mục lưu và đọc file log cho trang Logs
icon_set = "nerd"            # "ascii", "unicode", hoặc "nerd"

[docker]
default_conn_type = "socket" # "socket" (mặc định), "local", hoặc "http"
# addr_or_path = "/var/run/docker.sock"
timeout = 120                # Thời gian chờ kết nối (giây)
# client_version = "1.45"    # (Tùy chọn) Ép phiên bản Docker API

[ui]
frame_rate = 30.0
tick_rate = 4.0
```

### Cấu hình cho từng Container Engine CLI thay thế Docker Desktop

Bạn chỉ cần chỉnh mục `[docker]` trong `.config/config.toml` tương ứng với engine CLI mà bạn đang sử dụng:

#### 5.1. Docker Engine CLI thuần trên Linux (`dockerd`)
- **Chế độ mặc định (Rootful)**:
  ```toml
  [docker]
  default_conn_type = "socket"
  addr_or_path = "/var/run/docker.sock"
  timeout = 120
  ```
- **Chế độ Rootless Docker**:
  Kiểm tra `echo $XDG_RUNTIME_DIR/docker.sock` (thường là `/run/user/1000/docker.sock`):
  ```toml
  [docker]
  default_conn_type = "socket"
  addr_or_path = "/run/user/1000/docker.sock"
  timeout = 120
  ```

#### 5.2. Podman CLI (`podman`)
Podman cung cấp REST API tương thích với Docker Engine qua `podman.socket`.
- **Trên Linux (Rootless Podman)**:
  1. Bật socket của Podman:
     ```bash
     systemctl --user enable --now podman.socket
     ```
  2. Cấu hình trong `config.toml`:
     ```toml
     [docker]
     default_conn_type = "socket"
     addr_or_path = "/run/user/1000/podman/podman.sock"
     timeout = 120
     ```
- **Trên macOS / Windows (Podman Machine)**:
  1. Lấy đường dẫn socket bằng lệnh:
     ```bash
     podman machine inspect --format '{{.ConnectionInfo.PodmanSocket.Path}}'
     ```
  2. Điền đường dẫn đó vào `addr_or_path` với `default_conn_type = "socket"` (hoặc `"local"` trên Windows Named Pipe).

#### 5.3. Colima CLI (`colima`)
1. Khởi chạy Colima: `colima start`
2. Xem đường dẫn socket bằng `colima status` (thường nằm ở `~/.colima/default/docker.sock`):
   ```toml
   [docker]
   default_conn_type = "socket"
   addr_or_path = "/home/<user>/.colima/default/docker.sock" # hoặc /Users/<user>/.colima/default/docker.sock trên macOS
   timeout = 120
   ```

#### 5.4. OrbStack CLI (macOS)
```toml
[docker]
default_conn_type = "socket"
addr_or_path = "/Users/<user>/.orbstack/run/docker.sock"
timeout = 120
```

#### 5.5. Rancher Desktop (chế độ `dockerd` / `moby`)
```toml
[docker]
default_conn_type = "socket"
addr_or_path = "/Users/<user>/.rd/docker.sock" # Trên Linux: /home/<user>/.rd/docker.sock
timeout = 120
```

#### 5.6. Kết nối qua HTTP / TCP Daemon (Remote Engine hoặc `podman system service`)
Khi chạy daemon qua cổng mạng (ví dụ `dockerd -H tcp://127.0.0.1:2375` hoặc `podman system service -t 0 tcp:127.0.0.1:8080`):
```toml
[docker]
default_conn_type = "http"
addr_or_path = "tcp://127.0.0.1:2375" # Hỗ trợ tcp://, http://, hoặc https://
timeout = 120
```

---

## 6. Kiến trúc & Phong cách Code (Coding Style)

Dự án được thiết kế theo hướng **Module hóa (Modular Architecture)**, đảm bảo dễ dàng kế thừa, mở rộng và bảo trì:

```text
src/
├── main.rs                      # Entrypoint: khởi tạo CLI, AppConfig, Logging, DockerConnection, AppServices
├── tuikk_core/                  # Lõi hệ thống: App event loop, Router, Config, KeyMap, ThemeColor, Tui
├── shared/                      # Thành phần dùng chung
│   ├── base/                    # Trait nền tảng: Page, BaseWidget, Action, BaseFilter, BaseErr, Icons
│   ├── helpers/                 # Tiện ích mở rộng (logging, bảng màu button)
│   └── widgets/                 # Widget dùng chung toàn cục (SideBar, Button, StatusBar)
├── modules/                     # Các module tính năng độc lập
│   ├── dashboard/               # Module Dashboard (biểu đồ CPU, RAM, Storage)
│   ├── containers/              # Module Containers (List, Detail, Lifecycle actions)
│   ├── images/                  # Module Images (List, Detail, Run Image + .env parser)
│   └── logs/                    # Module Logs (xem, lọc mức độ và xóa/clear file log)
└── tests/                       # Unit tests tách riêng cho từng module
```

### Nguyên tắc tổ chức bên trong mỗi Module (`src/modules/<feature>/`)
1. **Phân tách trách nhiệm rõ ràng**:
   - `actions.rs`: Định nghĩa các `Action` / `ButtonAction` riêng của module.
   - `constants.rs`: Định nghĩa các enum trạng thái tĩnh như `Sort`, `Tab`, `Field`.
   - `*_dto.rs`: Chứa các cấu trúc dữ liệu (DTO) và các hàm thuần túy (`pure functions`) để parse/format dữ liệu (ví dụ: `parse_env_content`, `parse_repo_tag`, `parse_log_line`, `format_bytes`).
   - `*_service.rs`: Chứa logic gọi API bất đồng bộ (`bollard` hoặc filesystem) và logic lọc/sắp xếp dữ liệu. Được khởi tạo một lần và chia sẻ qua `AppServices` (`Arc<Service>`).
   - `pages.rs`, `list_page.rs`, `detail_page.rs`: Thực thi trait `Page`, quản lý state, điều hướng nội bộ và xử lý sự kiện phím/chuột.
2. **Hàm `draw()` ngắn gọn, tách nhỏ các file UI con (`ui/*.rs`)**:
   - Không viết logic render dài dòng bên trong `list_page.rs` hay `detail_page.rs`.
   - Mỗi thành phần giao diện được tách thành một file nhỏ trong thư mục `ui/` của module đó (ví dụ: `ui/search_box.rs`, `ui/table.rs`, `ui/action_column.rs`, `ui/detail_view.rs`, `ui/run_view.rs`, `ui/cpu_chart.rs`, `ui/mem_chart.rs`, `ui/storage_panel.rs`).
   - Hàm `Page::draw()` chỉ tính toán `Layout` và ủy quyền vẽ cho các hàm `ui::<component>::draw(...)`.
3. **Dễ dàng kiểm thử (Testability)**:
   - Mỗi module trong `src/modules/` đều đi kèm một file unit test tương ứng trong `src/tests/` (`dashboard_test.rs`, `containers_test.rs`, `images_test.rs`, `logs_test.rs`, `click_tracker_test.rs`).
