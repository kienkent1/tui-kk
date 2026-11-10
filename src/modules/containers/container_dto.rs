use std::collections::BTreeSet;

use bollard::plugin::{ContainerSummary, PortSummary, PortSummaryTypeEnum};
use ratatui::layout::Rect;

use crate::modules::containers::actions::ButtonAction;

pub struct ContainerDto {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RowTarget {
    pub row: usize,
    pub action: ButtonAction,
}

#[derive(Clone, Debug)]
pub struct RowButton {
    pub target: RowTarget,
    pub rect: Rect,
}

pub fn short_container_id(c: &ContainerSummary) -> &str {
    c.id.as_deref()
        .map(|s| &s[..s.len().min(12)])
        .unwrap_or("-")
}

pub fn container_display_name(c: &ContainerSummary) -> &str {
    c.names
        .as_ref()
        .and_then(|n| n.first())
        .map(|s| s.trim_start_matches('/'))
        .unwrap_or("-")
}

/// Build a compact, deduplicated port string for the PORTS column.
/// Entries are keyed by `(public, private, protocol)`; IPv4/IPv6 duplicates collapse.
pub fn format_ports(ports: Option<&[PortSummary]>) -> String {
    let Some(list) = ports else {
        return String::new();
    };

    let unique: BTreeSet<(Option<u16>, u16, &str)> = list
        .iter()
        .map(|p| {
            let proto = match p.typ {
                Some(PortSummaryTypeEnum::UDP) => "udp",
                Some(PortSummaryTypeEnum::SCTP) => "sctp",
                _ => "tcp",
            };
            (p.public_port, p.private_port, proto)
        })
        .collect();

    unique
        .into_iter()
        .map(|(public, private, proto)| match public {
            Some(pub_p) => format!("{proto}/{pub_p}:{private}"),
            None => format!("{proto}/{private}"),
        })
        .collect::<Vec<_>>()
        .join(", ")
}
