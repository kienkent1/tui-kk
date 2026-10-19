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
