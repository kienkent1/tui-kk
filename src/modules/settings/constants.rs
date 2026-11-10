#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsTab {
    #[default]
    General,
    Docker,
    Keybindings,
}

impl SettingsTab {
    pub const ALL: [Self; 3] = [Self::General, Self::Docker, Self::Keybindings];

    pub fn title(self) -> &'static str {
        match self {
            Self::General => "1. General & UI",
            Self::Docker => "2. Docker Engine",
            Self::Keybindings => "3. Keybindings",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::General => Self::Docker,
            Self::Docker => Self::Keybindings,
            Self::Keybindings => Self::General,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::General => Self::Keybindings,
            Self::Docker => Self::General,
            Self::Keybindings => Self::Docker,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GeneralField {
    #[default]
    Theme,
    IconSet,
    QueryMode,
    LogLevel,
    LogDir,
    FrameRate,
    TickRate,
}

impl GeneralField {
    pub const ALL: [Self; 7] = [
        Self::Theme,
        Self::IconSet,
        Self::QueryMode,
        Self::LogLevel,
        Self::LogDir,
        Self::FrameRate,
        Self::TickRate,
    ];

    pub fn next(self) -> Self {
        match self {
            Self::Theme => Self::IconSet,
            Self::IconSet => Self::QueryMode,
            Self::QueryMode => Self::LogLevel,
            Self::LogLevel => Self::LogDir,
            Self::LogDir => Self::FrameRate,
            Self::FrameRate => Self::TickRate,
            Self::TickRate => Self::Theme,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Theme => Self::TickRate,
            Self::IconSet => Self::Theme,
            Self::QueryMode => Self::IconSet,
            Self::LogLevel => Self::QueryMode,
            Self::LogDir => Self::LogLevel,
            Self::FrameRate => Self::LogDir,
            Self::TickRate => Self::FrameRate,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DockerField {
    #[default]
    ConnectionType,
    AddrOrPath,
    Timeout,
    ClientVersion,
}

impl DockerField {
    pub const ALL: [Self; 4] = [
        Self::ConnectionType,
        Self::AddrOrPath,
        Self::Timeout,
        Self::ClientVersion,
    ];

    pub fn next(self) -> Self {
        match self {
            Self::ConnectionType => Self::AddrOrPath,
            Self::AddrOrPath => Self::Timeout,
            Self::Timeout => Self::ClientVersion,
            Self::ClientVersion => Self::ConnectionType,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::ConnectionType => Self::ClientVersion,
            Self::AddrOrPath => Self::ConnectionType,
            Self::Timeout => Self::AddrOrPath,
            Self::ClientVersion => Self::Timeout,
        }
    }
}

