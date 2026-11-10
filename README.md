# TUI-KK 🐳

**`tui-kk`** is a fast, modular **Terminal User Interface (TUI)** written in **Rust** (`ratatui`, `bollard`, `tokio`, `crossterm`) for managing and monitoring containers directly from your terminal.

Because it communicates directly with the **Docker Engine API** over Unix Sockets, Windows Named Pipes, or HTTP/TCP, `tui-kk` works seamlessly with **Docker Engine CLI (`dockerd`)**, **Podman**, **Colima**, **OrbStack**, **Rancher Desktop**, and **Docker Desktop**.

---

## 1. What Does `tui-kk` Do? (Core Features)

The interface features a persistent **Sidebar** on the left (supporting both keyboard navigation and mouse clicks) and 5 main modules:

1. **Dashboard (`1. Dashboard`)**
   - **Engine Overview Cards**: Docker Engine version, OS/Architecture, CPU cores, total host RAM, and container/image counts.
   - **Real-Time Charts**: **Gauge** and **Braille Line Charts** tracking aggregate **CPU (%)** and **RAM Usage** across running containers.
   - **Storage Breakdown**: **BarChart** comparing disk usage across `Images`, `Containers`, and `Volumes`, paired with a per-container resource table (`CPU %`, `MEM USAGE / LIMIT`, `MEM %`, `STORAGE RW / ROOTFS`).
2. **Containers (`2. Containers`)**
   - Search (`local` in-memory or `remote` API mode), filter, and sort by `Name`, `Image`, `State`, or `Created`.
   - Lifecycle management via keyboard shortcuts or interactive mouse buttons in the **Action** column: `Start (▶)`, `Stop (■)`, `Restart`, `Remove (🗑)`.
   - **Container Detail View** with 4 tabs: `1. Overview`, `2. Network & Mounts`, `3. Env`, and `4. Raw JSON`.
3. **Images (`3. Images`)**
   - Search, sort (`RepoTag`, `Id`, `Size`, `Created`, `Containers`), remove images (`Remove`), and prune dangling images (`Prune`).
   - **Docker Desktop-Style Run Image (`ImageRunPage`)**:
     - Configure **Container Name**, **Port Mappings** (`host:container[/proto]`), and **Volume Mounts** (`host:container[:ro]`).
     - **Parse `.env` Files**: specify a `.env` file path to automatically parse and load environment variables (supports `#` comments, `export ` prefixes, single/double quotes `'...'` / `"..."` with escape sequences, and duplicate key merging).
     - **Interactive Environment Variables Builder**: manually add, edit, or delete `KEY=VALUE` entries, paste multi-line `.env` blocks directly, or import default `Env` variables defined in the image (`Import Image Env`).
   - **Image Detail View** with 4 tabs: `1. Overview`, `2. Layers`, `3. Config & Env`, and `4. Raw JSON`.
4. **Logs (`4. Logs`)**
   - Browse application and engine error/warning log files from the user-configured `log_dir`.
   - Displays file size, last modified timestamp, total lines, and `ERROR` / `WARN` counts per file.
   - **Log Detail View** with 4 level filter tabs (`1: All`, `2: Errors Only`, `3: Warnings + Errors`, `4: Raw Lines`).
   - Supports **truncating/clearing (`Clear`)**, **deleting individual log files (`Delete`)**, or **deleting all log files (`Delete All`)**.
5. **Settings (`5. Settings`)**
   - View, edit, and save `config.toml` settings (`Theme`, `Icon Set`, `Query Mode`, `Log Level`, `Log Dir`, `Frame Rate`, `Tick Rate`).
   - Configure and test (`Ping`) the **Docker Engine** connection (`socket`, `local`, `http`, `addr_or_path`, `timeout`, `client_version`).
   - Inspect all active **Keybindings** by scope.

---

## 2. Installation & Download

### Prerequisites
- **Rust Toolchain**: `>= 1.88.0` (Rust 2024 Edition).
- **Container Engine**: A running Docker-compatible daemon (`dockerd`, `podman.socket`, Colima, OrbStack, Rancher Desktop, or Docker Desktop).
- *(Optional)* **Nerd Font** (e.g., *JetBrainsMono Nerd Font*) when using `icon_set = "nerd"`.

### Clone & Build

```bash
# 1. Clone the repository
git clone <repository-url>
cd tui-kk

# 2. Build in debug mode
cargo build

# 3. Build optimized release binary
cargo build --release

# 4. (Optional) Install the binary into ~/.cargo/bin
cargo install --path .
```

---

## 3. Running & Keybindings

### CLI Commands

```bash
# Run with default configuration
cargo run

# Run optimized release build
cargo run --release

# Override UI frame rate (FPS) and tick rate (Hz) (valid range: 1.0–240.0)
cargo run -- --frame-rate 60 --tick-rate 4

# Preview ASCII vs. Nerd Font icons in your terminal
cargo run -- --icon-preview

# Display CLI help and version
cargo run -- --help
cargo run -- --version
```

### Default Keybindings

| Key | Action | Scope |
| :--- | :--- | :--- |
| `Tab` / `Shift+Tab` | Navigate to next / previous page in the Sidebar | Global |
| `j` / `↓`, `k` / `↑` | Move selection cursor down / up or scroll | Global |
| `Enter` | Open detail view / Confirm action | Global |
| `Esc` | Go back to list / Cancel input mode | Global |
| `/` | Focus search bar | Containers, Images, Logs |
| `s` / `o` | Cycle sort column (`CycleSort`) / Toggle sort order (`ToggleOrder`) | Containers, Images, Logs |
| `F5` | Refresh current view | Global |
| `s` / `Ctrl+R` / `d` | Start-Stop / Restart / Remove container | Containers |
| `r` | Open **Run Image** form (create container & parse `.env`) | Images |
| `p` / `d` | Prune dangling images / Remove selected image | Images |
| `c` / `d` / `D` | Clear (truncate) log file / Delete log file / Delete all logs | Logs |
| `Ctrl+S` / `t` | Save configuration / Test (`Ping`) Docker connection | Settings |
| `l` | Jump directly to the **Logs** page | Global |
| `Shift+M` | Toggle mouse capture on / off | Global |
| `q` / `Ctrl+C` | Quit application | Global |

---

## 4. Running Tests

Unit tests are organized by module inside [`src/tests/`](src/tests/):

```bash
# Run the entire test suite
cargo test

# Run tests with stdout output visible
cargo test -- --nocapture

# Run tests for a specific module
cargo test dashboard_test
cargo test containers_test
cargo test images_test
cargo test logs_test
cargo test settings_test
cargo test click_tracker_test
```

---

## 5. Configuration Guide for Alternative Container Engines (Without Docker Desktop)

`tui-kk` loads its configuration from:
1. **Local workspace override (highest priority)**: `.config/config.toml` in the current working directory.
2. **User system config directory**:
   - **Linux / macOS**: `~/.config/tuikk/config.toml`
   - **Windows**: `C:\Users\<User>\AppData\Roaming\tuikk\config.toml`

### Base `config.toml` Structure

```toml
theme = "dark"               # "dark", "light", or a custom theme name in theme.json5
log_level = "warn"           # "trace", "debug", "info", "warn", "error"
query_mode = "local"         # "local" (in-memory filtering) or "remote" (daemon API filtering)
log_dir = "./logs"           # Directory where application logs are stored and read by the Logs page
icon_set = "nerd"            # "ascii", "unicode", or "nerd"

[docker]
default_conn_type = "socket" # "socket" (default), "local", or "http"
# addr_or_path = "/var/run/docker.sock"
timeout = 120                # Request timeout in seconds
# client_version = "1.45"    # (Optional) Pin Docker Engine API version

[ui]
frame_rate = 30.0
tick_rate = 4.0
```

### Configuring Alternative Container Engines

Update the `[docker]` section in `config.toml` to match your container engine CLI:

#### 5.1. Standalone Docker Engine CLI on Linux (`dockerd`)
- **Standard (Rootful) Daemon**:
  ```toml
  [docker]
  default_conn_type = "socket"
  addr_or_path = "/var/run/docker.sock"
  timeout = 120
  ```
- **Rootless Docker Daemon**:
  Check `echo $XDG_RUNTIME_DIR/docker.sock` (typically `/run/user/1000/docker.sock`):
  ```toml
  [docker]
  default_conn_type = "socket"
  addr_or_path = "/run/user/1000/docker.sock"
  timeout = 120
  ```

#### 5.2. Podman CLI (`podman`)
Podman exposes a Docker-compatible API socket via `podman.socket`.
- **Linux (Rootless Podman)**:
  1. Enable the user Podman socket:
     ```bash
     systemctl --user enable --now podman.socket
     ```
  2. Configure `config.toml`:
     ```toml
     [docker]
     default_conn_type = "socket"
     addr_or_path = "/run/user/1000/podman/podman.sock"
     timeout = 120
     ```
- **macOS / Windows (Podman Machine)**:
  1. Inspect the machine socket path:
     ```bash
     podman machine inspect --format '{{.ConnectionInfo.PodmanSocket.Path}}'
     ```
  2. Set `addr_or_path` to that path with `default_conn_type = "socket"` (or `"local"` on Windows Named Pipe).

#### 5.3. Colima CLI (`colima`)
1. Start Colima: `colima start`
2. Point `addr_or_path` to Colima's socket (`~/.colima/default/docker.sock`):
   ```toml
   [docker]
   default_conn_type = "socket"
   addr_or_path = "/Users/<username>/.colima/default/docker.sock" # or /home/<username>/.colima/default/docker.sock on Linux
   timeout = 120
   ```

#### 5.4. OrbStack CLI (macOS)
```toml
[docker]
default_conn_type = "socket"
addr_or_path = "/Users/<username>/.orbstack/run/docker.sock"
timeout = 120
```

#### 5.5. Rancher Desktop (`dockerd` / `moby` runtime)
```toml
[docker]
default_conn_type = "socket"
addr_or_path = "/Users/<username>/.rd/docker.sock" # Linux: /home/<username>/.rd/docker.sock
timeout = 120
```

#### 5.6. Remote HTTP / TCP Daemon (`dockerd -H tcp://...` or `podman system service`)
When connecting to an engine listening on TCP/HTTP (e.g., `dockerd -H tcp://127.0.0.1:2375` or `podman system service -t 0 tcp:127.0.0.1:8080`):
```toml
[docker]
default_conn_type = "http"
addr_or_path = "tcp://127.0.0.1:2375" # Supports tcp://, http://, or https://
timeout = 120
```

---

## 6. Architecture & Coding Style

The codebase follows a strict **Modular Architecture** designed for readability, inheritance/reuse, and easy maintenance:

```text
src/
├── main.rs                      # Entrypoint: CLI parsing, AppConfig, Logging, DockerConnection, AppServices
├── tuikk_core/                  # Core infrastructure: App loop, Router, Config, KeyMap, ThemeColor, Tui
├── shared/                      # Shared abstractions & reusable widgets
│   ├── base/                    # Base traits & types: Page, BaseWidget, Action, BaseFilter, BaseErr, Icons
│   ├── helpers/                 # Logging extensions & button color palettes
│   └── widgets/                 # Global widgets (SideBar, Button, StatusBar)
├── modules/                     # Feature modules
│   ├── dashboard/               # CPU, RAM, and Storage charts + per-container metrics
│   ├── containers/              # Container list, lifecycle actions, and tabbed inspect view
│   ├── images/                  # Image list, tabbed inspect view, and Run Image (.env parser + env builder)
│   ├── logs/                    # Log file browser, level filter tabs, clear & delete actions
│   └── settings/                # Interactive configuration editor, Docker connection tester, keybindings viewer
└── tests/                       # Dedicated unit test module per feature
```

### Module Design Principles (`src/modules/<feature>/`)
1. **Clear Separation of Concerns**:
   - `actions.rs`: Module-scoped `Action` and `ButtonAction` enums dispatched through the central event channel.
   - `constants.rs`: Strongly typed enums for sorting (`*Sort`), tabs (`*Tab`), and form focus fields (`*Field`).
   - `*_dto.rs`: Data Transfer Objects and pure formatting/parsing functions (e.g., `parse_env_content`, `parse_repo_tag`, `parse_log_line`, `format_bytes`).
   - `*_service.rs`: Asynchronous service layer wrapping `bollard` or filesystem I/O, registered in `AppServices` via `Arc<Service>`.
   - `pages.rs`, `list_page.rs`, `detail_page.rs`: Implement the `Page` trait, managing state transitions, async request sequencing (`req_seq`), and keyboard/mouse events.
2. **Short `draw()` Methods & Small Embedded UI Files (`ui/*.rs`)**:
   - `Page::draw()` methods never contain monolithic widget-building blocks.
   - Instead, each visual area is extracted into a small, focused file inside the module's `ui/` directory (e.g., `ui/search_box.rs`, `ui/table.rs`, `ui/action_column.rs`, `ui/detail_view.rs`, `ui/run_view.rs`, `ui/cpu_chart.rs`, `ui/mem_chart.rs`, `ui/storage_panel.rs`).
   - `Page::draw()` only computes the high-level `Layout` rects and delegates rendering to `ui::<sub_module>::draw(...)`.
3. **First-Class Unit Testing (`src/tests/`)**:
   - Every module in `src/modules/` has a corresponding unit test file in `src/tests/` testing pure business logic, parsers, sorting/filtering comparators, and page state transitions without requiring a live Docker daemon.
