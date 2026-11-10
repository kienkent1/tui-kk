#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageSort {
    RepoTag,
    Id,
    Size,
    Created,
    Containers,
}

impl ImageSort {
    /// "repotag" | "repo" | "name" => (RepoTag, false [asc]), "-size" => (Size, true [desc])
    pub fn parse(s: &str) -> Option<(Self, bool)> {
        let s = s.trim();
        let (desc, s) = match s.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, s),
        };
        let key = match s.to_lowercase().as_str() {
            "repotag" | "repo" | "name" | "repository" => Self::RepoTag,
            "id" => Self::Id,
            "size" => Self::Size,
            "created" => Self::Created,
            "containers" => Self::Containers,
            _ => return None,
        };
        Some((key, desc))
    }

    pub fn next(self) -> Self {
        match self {
            Self::RepoTag => Self::Id,
            Self::Id => Self::Size,
            Self::Size => Self::Created,
            Self::Created => Self::Containers,
            Self::Containers => Self::RepoTag,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::RepoTag => "repo",
            Self::Id => "id",
            Self::Size => "size",
            Self::Created => "created",
            Self::Containers => "containers",
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
pub enum ImageDetailTab {
    #[default]
    Overview,
    Layers,
    Config,
    RawJson,
}

impl ImageDetailTab {
    pub const ALL: [Self; 4] = [Self::Overview, Self::Layers, Self::Config, Self::RawJson];

    pub fn title(self) -> &'static str {
        match self {
            Self::Overview => "1. Overview",
            Self::Layers => "2. Layers",
            Self::Config => "3. Config & Env",
            Self::RawJson => "4. Raw JSON",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Overview => Self::Layers,
            Self::Layers => Self::Config,
            Self::Config => Self::RawJson,
            Self::RawJson => Self::Overview,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Overview => Self::RawJson,
            Self::Layers => Self::Overview,
            Self::Config => Self::Layers,
            Self::RawJson => Self::Config,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RunField {
    #[default]
    ContainerName,
    Ports,
    Volumes,
    EnvFilePath,
    EnvKey,
    EnvValue,
    EnvList,
    RunButton,
}

impl RunField {
    pub const ALL: [Self; 8] = [
        Self::ContainerName,
        Self::Ports,
        Self::Volumes,
        Self::EnvFilePath,
        Self::EnvKey,
        Self::EnvValue,
        Self::EnvList,
        Self::RunButton,
    ];

    pub fn next(self) -> Self {
        match self {
            Self::ContainerName => Self::Ports,
            Self::Ports => Self::Volumes,
            Self::Volumes => Self::EnvFilePath,
            Self::EnvFilePath => Self::EnvKey,
            Self::EnvKey => Self::EnvValue,
            Self::EnvValue => Self::EnvList,
            Self::EnvList => Self::RunButton,
            Self::RunButton => Self::ContainerName,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::ContainerName => Self::RunButton,
            Self::Ports => Self::ContainerName,
            Self::Volumes => Self::Ports,
            Self::EnvFilePath => Self::Volumes,
            Self::EnvKey => Self::EnvFilePath,
            Self::EnvValue => Self::EnvKey,
            Self::EnvList => Self::EnvValue,
            Self::RunButton => Self::EnvList,
        }
    }
}
