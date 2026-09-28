#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Router {
    Dashboard,
    Containers,
    Images,
    Settings,
    Logs,
}

impl Router {
    pub const ALL: [Self; 5] = [Self::Dashboard, Self::Containers, Self::Images, Self::Settings, Self::Logs];

    #[inline]
    pub fn next(self) -> Self {
        Self::ALL[(self as usize + 1) % Self::ALL.len()]
    }
    #[inline]
    pub fn prev(self) -> Self {
        Self::ALL[(self as usize + Self::ALL.len() - 1) % Self::ALL.len()]
    }
}