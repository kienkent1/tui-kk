use std::{
    fs,
    path::{Path, PathBuf},
};

use bollard::models::ImageSummary;
use ratatui::layout::Rect;
use serde::{Deserialize, Serialize};

use crate::modules::{dashboard::dashboard_dto::format_bytes, images::actions::ImageButtonAction};

pub struct ImageDto {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageRowTarget {
    pub row: usize,
    pub action: ImageButtonAction,
}

#[derive(Clone, Debug)]
pub struct ImageRowButton {
    pub target: ImageRowTarget,
    pub rect: Rect,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvVarEntry {
    pub key: String,
    pub value: String,
    pub source: String,
}

impl EnvVarEntry {
    pub fn new(key: impl Into<String>, value: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            source: source.into(),
        }
    }

    pub fn to_docker_env(&self) -> String {
        format!("{}={}", self.key, self.value)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunImageRequest {
    pub image: String,
    pub container_name: Option<String>,
    /// Pairs of `(host_port, container_port_with_proto)` e.g. `("8080", "80/tcp")`
    pub ports: Vec<(String, String)>,
    /// Volume binds e.g. `"/host/path:/container/path:ro"`
    pub volumes: Vec<String>,
    pub env_vars: Vec<EnvVarEntry>,
}

/// Strip optional `sha256:` prefix and take up to 12 characters.
pub fn short_image_id(id: &str) -> &str {
    let stripped = id.strip_prefix("sha256:").unwrap_or(id);
    &stripped[..stripped.len().min(12)]
}

/// Extract `(repository, tag)` from the first entry in `repo_tags`.
/// Properly handles registry host ports such as `localhost:5000/app:1.0` -> `("localhost:5000/app", "1.0")`.
pub fn parse_repo_tag(img: &ImageSummary) -> (String, String) {
    let Some(first) = img.repo_tags.first() else {
        return ("<none>".to_owned(), "<none>".to_owned());
    };
    let s = first.trim();
    if s.is_empty() || s == "<none>:<none>" {
        return ("<none>".to_owned(), "<none>".to_owned());
    }

    // Find the last ':' that occurs after the last '/'
    let last_slash = s.rfind('/');
    if let Some(colon_idx) = s.rfind(':') {
        let is_tag_sep = match last_slash {
            Some(slash_idx) => colon_idx > slash_idx,
            None => true,
        };
        if is_tag_sep {
            let repo = &s[..colon_idx];
            let tag = &s[colon_idx + 1..];
            return (
                if repo.is_empty() { "<none>" } else { repo }.to_owned(),
                if tag.is_empty() { "latest" } else { tag }.to_owned(),
            );
        }
    }

    (s.to_owned(), "latest".to_owned())
}

/// Return the best human-readable reference to run a container from `img`:
/// prefers `"repo:tag"` if tagged, otherwise falls back to `short_image_id`.
pub fn primary_image_ref(img: &ImageSummary) -> String {
    let (repo, tag) = parse_repo_tag(img);
    if repo != "<none>" && tag != "<none>" {
        format!("{repo}:{tag}")
    } else if repo != "<none>" {
        repo
    } else {
        short_image_id(&img.id).to_owned()
    }
}

pub fn format_image_size(bytes: i64) -> String {
    if bytes <= 0 {
        return "0 B".to_owned();
    }
    format_bytes(bytes as u64)
}

/// Parse `.env` file content into a list of `EnvVarEntry` items.
/// Supports:
/// - Empty lines and `#` comment lines
/// - Optional `export ` prefix (`export KEY=VALUE`)
/// - Single-quoted (`'...'`), double-quoted (`"..."` with `\n`, `\"`, `\\` unescaping), and unquoted values
/// - Inline `# comments` on unquoted values
pub fn parse_env_content(content: &str) -> Vec<EnvVarEntry> {
    parse_env_content_with_source(content, ".env")
}

pub fn parse_env_content_with_source(content: &str, source: &str) -> Vec<EnvVarEntry> {
    let mut result = Vec::new();

    for raw_line in content.lines() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let line = trimmed
            .strip_prefix("export ")
            .map(str::trim_start)
            .unwrap_or(trimmed);

        let Some((raw_key, raw_val)) = line.split_once('=') else {
            continue;
        };

        let key = raw_key.trim();
        if key.is_empty() || key.contains(char::is_whitespace) {
            continue;
        }

        let val_trimmed = raw_val.trim();
        let value = if val_trimmed.len() >= 2
            && val_trimmed.starts_with('"')
            && val_trimmed.ends_with('"')
        {
            let inner = &val_trimmed[1..val_trimmed.len() - 1];
            unescape_double_quoted(inner)
        } else if val_trimmed.len() >= 2
            && val_trimmed.starts_with('\'')
            && val_trimmed.ends_with('\'')
        {
            val_trimmed[1..val_trimmed.len() - 1].to_owned()
        } else {
            // Strip inline comment ` # ...` on unquoted value
            let without_comment = match val_trimmed.find(" #") {
                Some(idx) => val_trimmed[..idx].trim_end(),
                None => val_trimmed,
            };
            without_comment.to_owned()
        };

        merge_env_vars(
            &mut result,
            vec![EnvVarEntry::new(key.to_owned(), value, source)],
        );
    }

    result
}

fn unescape_double_quoted(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('t') => out.push('\t'),
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Expand leading `~/` to the user's home directory if available.
pub fn expand_tilde_path(raw: &str) -> PathBuf {
    let trimmed = raw.trim();
    if let Some(rest) = trimmed.strip_prefix("~/") {
        if let Some(base) = directories::BaseDirs::new() {
            return base.home_dir().join(rest);
        }
    }
    PathBuf::from(trimmed)
}

/// Read and parse a `.env` file from disk.
pub fn parse_env_file(path: &Path) -> Result<Vec<EnvVarEntry>, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read '{}': {e}", path.display()))?;
    let source = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(".env");
    Ok(parse_env_content_with_source(&content, source))
}

/// Merge `incoming` environment variables into `target`, updating existing keys in place
/// and appending new keys in order.
pub fn merge_env_vars(target: &mut Vec<EnvVarEntry>, incoming: Vec<EnvVarEntry>) {
    for entry in incoming {
        if let Some(existing) = target.iter_mut().find(|e| e.key == entry.key) {
            existing.value = entry.value;
            existing.source = entry.source;
        } else {
            target.push(entry);
        }
    }
}

/// Parse comma- or whitespace-separated port mappings like `"8080:80, 5432:5432/tcp, 5353:53/udp"`
/// into `(host_port, container_port_with_proto)` pairs.
pub fn parse_port_mappings(input: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for token in input.split(|c: char| c == ',' || c.is_whitespace()) {
        let token = token.trim();
        if token.is_empty() {
            continue;
        }
        if let Some((host_p, cont_p)) = token.split_once(':') {
            let host_p = host_p.trim();
            let cont_p = cont_p.trim();
            if host_p.is_empty() || cont_p.is_empty() {
                continue;
            }
            let cont_with_proto = if cont_p.contains('/') {
                cont_p.to_owned()
            } else {
                format!("{cont_p}/tcp")
            };
            out.push((host_p.to_owned(), cont_with_proto));
        }
    }
    out
}

/// Parse comma-separated volume binds like `"/host/data:/data, my_vol:/var/lib/data:ro"`.
pub fn parse_volume_binds(input: &str) -> Vec<String> {
    input
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty() && s.contains(':'))
        .map(ToOwned::to_owned)
        .collect()
}
