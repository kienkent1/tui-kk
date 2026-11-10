#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogSort {
    Modified,
    Name,
    Size,
    Errors,
}

impl LogSort {
    /// "modified" => (Modified, false [asc]), "-errors" => (Errors, true [desc])
    pub fn parse(s: &str) -> Option<(Self, bool)> {
        let s = s.trim();
        let (desc, s) = match s.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, s),
        };
        let key = match s.to_lowercase().as_str() {
            "modified" | "time" | "date" => Self::Modified,
            "name" | "file" => Self::Name,
            "size" => Self::Size,
            "errors" | "error" => Self::Errors,
            _ => return None,
        };
        Some((key, desc))
    }

    pub fn next(self) -> Self {
        match self {
            Self::Modified => Self::Errors,
            Self::Errors => Self::Size,
            Self::Size => Self::Name,
            Self::Name => Self::Modified,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Modified => "modified",
            Self::Name => "name",
            Self::Size => "size",
            Self::Errors => "errors",
        }
    }

    pub fn to_filter_string(self, desc: bool) -> String {
        if desc {
            format!("-{}", self.as_str())
        } else {
            self.as_str().to_owned()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LogDetailTab {
    #[default]
    WarnAndError,
    ErrorsOnly,
    All,
    Raw,
}

impl LogDetailTab {
    pub const ALL: [Self; 4] = [
        Self::WarnAndError,
        Self::ErrorsOnly,
        Self::All,
        Self::Raw,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Self::WarnAndError => "1. Errors & Warnings",
            Self::ErrorsOnly => "2. Errors Only",
            Self::All => "3. All Logs",
            Self::Raw => "4. Raw",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::WarnAndError => Self::ErrorsOnly,
            Self::ErrorsOnly => Self::All,
            Self::All => Self::Raw,
            Self::Raw => Self::WarnAndError,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::WarnAndError => Self::Raw,
            Self::ErrorsOnly => Self::WarnAndError,
            Self::All => Self::ErrorsOnly,
            Self::Raw => Self::All,
        }
    }
}

