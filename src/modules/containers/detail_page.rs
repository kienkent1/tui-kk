use std::sync::Arc;

use bollard::plugin::ContainerInspectResponse;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    widgets::{Block, Paragraph},
};

use crate::{
    modules::containers::{actions::ContainerAction as CA, container_service::ContainerService},
    shared::{
        base::{
            base_action::Action,
            base_component::{Page, Tx},
        },
        helpers::ext_log::ResultExt,
    },
    tuikk_core::{
        key_map::{Command, KeyScope},
        themes::ThemeColor,
    },
};

pub struct ContainerDetailPage {
    id: String,
    svc: Arc<ContainerService>,
    tx: Tx,
    container: ContainerInspectResponse,

    loading: bool,
    error: Option<String>,
    theme_color: ThemeColor,
}

impl ContainerDetailPage {
    pub fn new(id: String, svc: Arc<ContainerService>, tx: Tx, theme_color: &ThemeColor) -> Self {
        Self {
            id: id,
            svc: svc,
            tx: tx,
            container: ContainerInspectResponse::default(),

            loading: false,
            error: None,
            theme_color: *theme_color,
        }
    }

    fn fetch(&mut self) {
        self.loading = true;
        self.error = None;
        let (svc, tx, id) = (self.svc.clone(), self.tx.clone(), self.id.clone());
        tokio::spawn(async move {
            let res = svc.get_container(&id).await.map_err(|e| e.to_string());
            let _ = tx.send(Action::Containers(CA::DetailLoaded(id, res)));
        });
    }
}
impl Page for ContainerDetailPage {
    fn scope(&self) -> KeyScope {
        KeyScope::Containers
    }
    fn on_activate(&mut self) {
        if self.container.id.is_none() {
            self.fetch();
        }
    }
    fn on_deactivate(&mut self) {}
    fn tick(&mut self) -> bool {
        true
    }
    fn captures_input(&self) -> bool {
        true
    }
    fn handle_command(&mut self, cmd: Command) -> Option<Action> {
        None
    }
    fn handle_key_event(&mut self, key: KeyEvent) -> Option<Action> {
        None
    }
    fn handle_mouse_event(&mut self, mouse: MouseEvent) -> Option<Action> {
        None
    }
    fn update(&mut self, action: &Action) -> bool {
        let Action::Containers(CA::DetailLoaded(id, res)) = action else {
            return false;
        };

        if *id != self.id {
            return false;
        }

        self.loading = false;

        match res {
            Ok(container) => {
                self.container = container.clone();
                self.error = None
            }
            Err(e) => self.error = Some(e.clone()),
        }

        true
    }
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let container = self.container.clone();
        let block = Block::bordered()
            .title(match &container.name {
                Some(name) => name.clone(),
                _ => String::from("Container"),
            })
            .border_style(Style::new().fg(self.theme_color.border));

        frame.render_widget(
            Paragraph::new(
                json5::to_string(&container)
                    .unwrap_or_else(|err| format!("Failed to serialize container: {err}")),
            )
            .style(Style::new().fg(self.theme_color.text))
            .block(block),
            area,
        );
    }
}
