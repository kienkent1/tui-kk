# Plan 03: Images Module

## 1. Objectives
- Implement the **Images** module (`src/modules/images/`) following the exact modular conventions and coding style of `containers`.
- Support core Docker Image management workflows:
  - **Image List (`ImageListPage`)**: local & remote search, filter, sort (`RepoTag`, `Id`, `Size`, `Created`, `Containers`), keyboard & mouse navigation.
  - **Image Actions**: run a new container from an image (`Run`), remove image (`Remove`) via action column buttons or keyboard shortcuts (`r` to run, `Command::Delete` to remove), and prune dangling images (`Prune`).
  - **Run Image (`ImageRunPage`)**: Docker Desktop-style container creation workflow supporting:
    - Container Name, Port mappings (`host:container[/proto]`), and Volume mounts (`host:container[:ro]`).
    - **Environment Variables management**:
      - Load & parse variables from a `.env` file (`parse_env_content` / `parse_env_file`, supporting comments, `export `, quoted values, and duplicate key merging).
      - Manually add/edit/delete `KEY=VALUE` environment variables interactively.
  - **Image Detail (`ImageDetailPage`)**: inspect image metadata (`ImageInspect`), layers (`RootFS.Layers`), config/env/exposed ports, and raw JSON with scrolling and tabs.
- Separate all rendering logic into small UI files under `src/modules/images/ui/`.
- Add unit tests in `src/tests/images_test.rs`.

---

## 2. Architecture & Module Structure

```mermaid
flowchart TD
    ImagePage["ImagePage (images/pages.rs)"] --> ImageListPage["ImageListPage (images/list_page.rs)"]
    ImagePage --> ImageDetailPage["ImageDetailPage (images/detail_page.rs)"]
    ImagePage --> ImageRunPage["ImageRunPage (images/run_page.rs)"]
    ImageListPage --> ImageService["ImageService (images/image_service.rs)"]
    ImageRunPage --> ImageService
    ImageListPage --> SearchBox["ui/search_box.rs"]
    ImageListPage --> ImageTable["ui/table.rs"]
    ImageListPage --> ActionColumn["ui/action_column.rs"]
    ImageDetailPage --> DetailView["ui/detail_view.rs"]
    ImageRunPage --> RunView["ui/run_view.rs"]
```

```text
src/
├── modules/images/
│   ├── mod.rs
│   ├── actions.rs                   # ImageAction & ImageButtonAction (Run, Remove)
│   ├── constants.rs                 # ImageSort, ImageDetailTab, RunField
│   ├── image_dto.rs                 # ImageRowTarget, EnvVarEntry, RunImageRequest, .env/port/volume parsers
│   ├── image_service.rs             # Bollard API: list_images, inspect_image, run_image, remove_image, prune_images
│   ├── error.rs                     # ImageError (#[extend_base_err])
│   ├── pages.rs                     # ImagePage (List <-> Detail <-> Run router)
│   ├── list_page.rs                 # ImageListPage (state, search, mouse/key events, delegates draw to ui/*)
│   ├── detail_page.rs               # ImageDetailPage (state, scroll, tabs, delegates draw to ui/*)
│   ├── run_page.rs                  # ImageRunPage (interactive form, .env loader, manual env editor, run container)
│   └── ui/
│       ├── mod.rs
│       ├── search_box.rs            # Search input bar + sort indicator
│       ├── table.rs                 # Image table (ID, REPOSITORY, TAG, SIZE, CONTAINERS, CREATED)
│       ├── action_column.rs         # Interactive Run (▶) & Remove (🗑) button column
│       ├── detail_view.rs           # Structured ImageInspect view (Overview, Layers, Config, Raw JSON)
│       └── run_view.rs              # Docker Desktop-style Run Container UI (Config, .env Loader, Env Builder & List)
└── tests/
    └── images_test.rs               # Unit tests for ImageService, .env parser, env merge, port/volume parser, sort, search
```

---

## 3. Detailed Technical Design

### 3.1. `ImageService` (`image_service.rs`)
- Registered in `AppServices` as `pub images: Arc<ImageService>`.
- Core methods:
  - `fetch_images(&self, filter: &BaseFilter, all: Option<bool>) -> Result<Vec<ImageSummary>, BaseErr>`
  - `get_images(&self, filter: BaseFilter, all: Option<bool>) -> Result<Vec<ImageSummary>, BaseErr>`
  - `query_images<'a>(&self, items: &'a [ImageSummary], filter: &BaseFilter) -> Vec<&'a ImageSummary>`
  - `get_image(&self, id: &str) -> Result<ImageInspect, BaseErr>`
  - `remove_image(&self, id: &str, force: bool) -> Result<(), BaseErr>`
  - `prune_images(&self) -> Result<u64, BaseErr>`
  - `run_image(&self, req: &RunImageRequest) -> Result<String, BaseErr>`:
    - Creates a container (`docker.create_container`) with container name, `Env` (`KEY=VALUE`), `ExposedPorts`, `HostConfig.PortBindings`, and `HostConfig.Binds`, then starts it (`docker.start_container`).

### 3.2. DTO & Formatting Helpers (`image_dto.rs`)
- `parse_repo_tag(image: &ImageSummary) -> (String, String)`:
  - Splits the primary tag in `repo_tags` on the last `:` (correctly handling registry ports such as `localhost:5000/my-app:1.0.0` -> `("localhost:5000/my-app", "1.0.0")`, and untagged images -> `("<none>", "<none>")`).
- `short_image_id(id: &str) -> &str`:
  - Strips the optional `"sha256:"` prefix and slices up to 12 characters.
- `format_image_size(bytes: i64) -> String`:
  - Converts byte count into human-readable units (`KB`, `MB`, `GB`).
- `EnvVarEntry { key, value, source }` & `RunImageRequest`:
  - `parse_env_content(content: &str) -> Vec<EnvVarEntry>`: parses `.env` text (supports comments `#`, `export KEY=VAL`, single/double quotes, inline comments on unquoted values).
  - `parse_env_file(path: &Path) -> Result<Vec<EnvVarEntry>, String>`: loads and parses a `.env` file from disk.
  - `merge_env_vars(target: &mut Vec<EnvVarEntry>, incoming: Vec<EnvVarEntry>)`: upserts environment variables by key.
  - `parse_port_mappings(input: &str) -> Vec<(String, String)>`: parses `host_port:container_port[/proto]` pairs.
  - `parse_volume_binds(input: &str) -> Vec<String>`: parses comma-separated volume binds.

### 3.3. Small Embedded UI Files (`src/modules/images/ui/`)
- `ImageListPage::draw()` splits the page into search bar, table, and action column (`▶` Run + `🗑` Remove), calling:
  - `ui::search_box::draw`
  - `ui::table::draw`
  - `ui::action_column::draw`
- `ImageDetailPage::draw()` calls `ui::detail_view::draw` to render the inspect view with tabs (`Overview`, `Layers`, `Config & Env`, `Raw JSON`) and line scrolling.
- `ImageRunPage::draw()` calls `ui::run_view::draw` to render the Docker Desktop-style container creation form (Container Name, Ports, Volumes, `.env` File Loader, Manual `KEY`/`VALUE` Builder, and interactive Environment Variables list).

---

## 4. Test Plan (`src/tests/images_test.rs`)
- `ImageSort::parse` & sorting comparators (`RepoTag`, `Id`, `Size`, `Created`, `Containers`).
- `ImageService::query_images`: case-insensitive search by repository, tag, or image ID; filter matching.
- `parse_repo_tag`: standard tags, registry host with port (`registry:5000/org/img:v2`), untagged images.
- `short_image_id`: stripping `sha256:` prefix and handling short strings safely.
- `parse_env_content` & `parse_env_file`: comments, `export` prefix, single/double quotes, inline comments, and temporary `.env` file loading.
- `merge_env_vars`, `parse_port_mappings`, and `parse_volume_binds`: upsert behavior and Docker port/volume binding generation.
