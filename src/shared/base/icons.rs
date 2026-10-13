use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum IconSet {
    #[default]
    Ascii,
    Unicode,
    Nerd,
}

#[derive(Debug, Clone, Copy)]
pub enum Icons {
    Remove,
    Start,
    Stop,
}

static ICON_SET: OnceLock<IconSet> = OnceLock::new();
impl Icons {
    /// Call once in `main` after loading config.
    /// A second call is ignored (OnceLock keeps the first value).
    pub fn init(icon_set: &IconSet) {
        if ICON_SET.set(*icon_set).is_err() {
            tracing::warn!("Icons::init called more than once; keeping the first value");
        }
    }

    /// Falls back to `IconSet::default()` (Ascii) if `init` was never called.
    fn icon_set() -> IconSet {
        ICON_SET.get().copied().unwrap_or_default()
    }

    pub fn glyph(self) -> &'static str {
        match (self, Self::icon_set()) {
            // ASCII: works everywhere
            (Icons::Remove, IconSet::Ascii) => "x",
            (Icons::Start, IconSet::Ascii) => ">",
            (Icons::Stop, IconSet::Ascii) => "#",

            // Plain Unicode: monochrome, so fg color applies
            (Icons::Remove, IconSet::Unicode) => "✕",
            (Icons::Start, IconSet::Unicode) => "▶",
            (Icons::Stop, IconSet::Unicode) => "■",

            // Nerd Font: use escapes, not pasted glyphs
            (Icons::Remove, IconSet::Nerd) => "\u{f1f8}", // trash
            (Icons::Start, IconSet::Nerd) => "\u{f04b}",  // play
            (Icons::Stop, IconSet::Nerd) => "\u{f04d}",   // stop
        }
    }
}
