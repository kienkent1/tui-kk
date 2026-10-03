#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerSort {
    Name,
    Image,
    State,
    Created,
}

impl ContainerSort {
    /// "name" => (Name, asc), "-created" => (Created, desc)
    /// true -> asc, false -> desc
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
}
