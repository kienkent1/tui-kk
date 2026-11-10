#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerSort {
    Name,
    Image,
    State,
    Created,
}

impl ContainerSort {
    /// "name" => (Name, true [asc]), "-created" => (Created, false [desc])
    /// Note: second tuple element is `desc` (`true` when prefixed with `-`).
    pub fn parse(s: &str) -> Option<(Self, bool)> {
        let s = s.trim();
        let (desc, s) = match s.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, s),
        };
        let key = match s.to_lowercase().as_str() {
            "name" => Self::Name,
            "image" => Self::Image,
            "state" | "status" => Self::State,
            "created" => Self::Created,
            _ => return None,
        };
        Some((key, desc))
    }

    pub fn next(self) -> Self {
        match self {
            Self::Name => Self::Image,
            Self::Image => Self::State,
            Self::State => Self::Created,
            Self::Created => Self::Name,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::Image => "image",
            Self::State => "state",
            Self::Created => "created",
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
pub enum ContainerDetailTab {
    #[default]
    Overview,
    NetworkMounts,
    Env,
    RawJson,
}

impl ContainerDetailTab {
    pub const ALL: [Self; 4] = [
        Self::Overview,
        Self::NetworkMounts,
        Self::Env,
        Self::RawJson,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Self::Overview => "1. Overview",
            Self::NetworkMounts => "2. Network & Mounts",
            Self::Env => "3. Env",
            Self::RawJson => "4. Raw JSON",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Overview => Self::NetworkMounts,
            Self::NetworkMounts => Self::Env,
            Self::Env => Self::RawJson,
            Self::RawJson => Self::Overview,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Overview => Self::RawJson,
            Self::NetworkMounts => Self::Overview,
            Self::Env => Self::NetworkMounts,
            Self::RawJson => Self::Env,
        }
    }
}
