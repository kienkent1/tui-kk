use bollard::models::ImageSummary;
use ratatui::layout::Rect;

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

pub fn format_image_size(bytes: i64) -> String {
    if bytes <= 0 {
        return "0 B".to_owned();
    }
    format_bytes(bytes as u64)
}
