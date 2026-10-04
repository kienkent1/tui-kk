use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{
    Frame,
    layout::{Rect, Size},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    shared::base::base_action::Action,
    tuikk_core::{
        key_map::{Command, KeyScope},
        routers::Router,
    },
};

pub type Tx = UnboundedSender<Action>;

/// `Component` is a trait that represents a visual and interactive element of the user interface.
///
/// Implementors of this trait can be registered with the main application loop and will be able to
/// receive events, update state, and be rendered on the screen.
pub trait Component {
    ///Sub-action for each component
    type Action;

    /// Register an action handler that can send actions for processing if necessary.
    ///
    /// # Arguments
    ///
    /// * `tx` - An unbounded sender that can send actions.
    ///
    /// # Returns
    ///
    /// * [`color_eyre::Result<()>`] - An Ok result or an error.
    /// fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> color_eyre::Result<()> {
    ///     let _ = tx; // to appease clippy
    ///     Ok(())
    /// }

    /// Register a configuration handler that provides configuration settings if necessary.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration settings.
    ///
    /// # Returns
    ///
    /// * [`color_eyre::Result<()>`] - An Ok result or an error.
    // fn register_config_handler(&mut self, config: Config) -> color_eyre::Result<()> {
    ///    let _ = config; // to appease clippy
    ///     Ok(())
    /// }
    /// Initialize the component with a specified area if necessary.
    ///
    /// # Arguments
    ///
    /// * `area` - Rectangular area to initialize the component within.
    ///
    /// # Returns
    ///
    /// * [`color_eyre::Result<()>`] - An Ok result or an error.

    /// fn init(&mut self, area: Size) -> color_eyre::Result<()> {
    ///     let _ = area; // to appease clippy
    ///     Ok(())
    /// }
    /// Handle incoming events and produce actions if necessary.
    ///
    /// # Arguments
    ///
    /// * `event` - An optional event to be processed.
    ///
    /// # Returns
    ///
    /// * [`color_eyre::Result<Option<Action>>`] - An action to be processed or none.

    /// fn handle_events(&mut self, event: Option<Event>) -> color_eyre::Result<Option<Action>> {
    ///     let action = match event {
    ///         Some(Event::Key(key_event)) => self.handle_key_event(key_event)?,
    ///         Some(Event::Mouse(mouse_event)) => self.handle_mouse_event(mouse_event)?,
    ///         _ => None,
    ///     };
    ///     Ok(action)
    /// }
    /// Handle key events and produce actions if necessary.
    ///
    /// # Arguments
    ///
    /// * `key` - A key event to be processed.
    ///
    /// # Returns
    ///
    /// * [`color_eyre::Result<Option<Action>>`] - An action to be processed or none.

    //============Custom func============
    ///Check keybinding, exp: "containers"
    fn scope(&self) -> KeyScope;

    /// Open component/page: load data, open stream.
    fn on_activate(&mut self);

    /// Leave page: cancel stream, export large volumes of data.
    fn on_deactivate(&mut self) {}

    ///Only the active page can receive it. Returns true if a redraw is required.
    fn tick(&mut self) -> bool {
        false
    }

    //===================================

    fn captures_input(&self) -> bool {
        false
    }

    fn handle_command(&mut self, cmd: Command) -> Option<Action> {
        let _ = cmd;
        None
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Option<Action> {
        let _ = key; // to appease clippy
        None
    }
    /// Handle mouse events and produce actions if necessary.
    ///
    /// # Arguments
    ///
    /// * `mouse` - A mouse event to be processed.
    ///
    /// # Returns
    ///
    /// * [`color_eyre::Result<Option<Action>>`] - An action to be processed or none.
    fn handle_mouse_event(&mut self, mouse: MouseEvent) -> Option<Action> {
        let _ = mouse; // to appease clippy
        None
    }
    /// Update the state of the component based on a received action. (REQUIRED)
    ///
    /// # Arguments
    ///
    /// * `action` - An action that may modify the state of the component.
    ///
    /// # Returns
    ///
    /// * [`color_eyre::Result<Option<Action>>`] - An action to be processed or none.
    fn update(&mut self, action: &Action) -> bool;
    /// Render the component on the screen. (REQUIRED)
    ///
    /// # Arguments
    ///
    /// * `f` - A frame used for rendering.
    /// * `area` - The area in which the component should be drawn.
    ///
    /// # Returns
    ///
    /// * [`color_eyre::Result<()>`] - An Ok result or an error.
    fn draw(&mut self, frame: &mut Frame, area: Rect);
}

pub trait Page {
    fn scope(&self) -> KeyScope;
    fn on_activate(&mut self) {}
    fn on_deactivate(&mut self) {}
    fn tick(&mut self) -> bool {
        false
    }
    fn captures_input(&self) -> bool;
    fn handle_command(&mut self, cmd: Command) -> Option<Action> {
        let _ = cmd;
        None
    }
    fn handle_key_event(&mut self, key: KeyEvent) -> Option<Action> {
        let _ = key;
        None
    }
    fn handle_mouse_event(&mut self, mouse: MouseEvent) -> Option<Action> {
        let _ = mouse;
        None
    }
    fn update(&mut self, action: &Action) -> bool {
        let _ = action;
        false
    }
    fn draw(&mut self, frame: &mut Frame, area: Rect);
}

impl<C: Component> Page for C {
    fn scope(&self) -> KeyScope {
        Component::scope(self)
    }
    fn on_activate(&mut self) {
        Component::on_activate(self)
    }
    fn on_deactivate(&mut self) {
        Component::on_deactivate(self)
    }
    fn tick(&mut self) -> bool {
        Component::tick(self)
    }
    fn captures_input(&self) -> bool {
        Component::captures_input(self)
    }
    fn handle_command(&mut self, cmd: Command) -> Option<Action> {
        Component::handle_command(self, cmd)
    }
    fn handle_key_event(&mut self, k: KeyEvent) -> Option<Action> {
        Component::handle_key_event(self, k)
    }
    fn handle_mouse_event(&mut self, m: MouseEvent) -> Option<Action> {
        Component::handle_mouse_event(self, m)
    }
    fn update(&mut self, a: &Action) -> bool {
        Component::update(self, a)
    }
    fn draw(&mut self, f: &mut Frame, r: Rect) {
        Component::draw(self, f, r)
    }
}

pub type PageBox = Box<dyn Page>;
