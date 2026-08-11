use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{Frame, backend::CrosstermBackend, layout::*, prelude::*, text::ToText, widgets::*};
use std::time::Duration;
use std::{
    collections::HashSet,
    io::{Stdout, stdout},
    rc::Rc,
    sync::Arc,
};
use tokio::sync::{Mutex, RwLock};
use tokio_util::sync::CancellationToken;

use crate::server::ControllerMap;

const LAVENDER: Color = Color::Rgb(150, 100, 175);

// Keeps track of what page the UI is currently on
#[derive(PartialEq, Copy, Clone)]
enum UIPage {
    Home,
    Controllers,
    Settings,
}

pub struct UIState {
    page: UIPage,
    selection_state: ListState,
    detected_controllers: Arc<RwLock<ControllerMap>>,
    active_controllers: Arc<RwLock<HashSet<u64>>>,
}

// This is an optimized version of the UI that can be run in the main thread and does not need to
// re-render on every single CPU cycle.
pub struct StarboardServerUI {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    ui_state: UIState,
    cancellation_token: CancellationToken,
}

impl StarboardServerUI {
    pub fn new(
        detected_controllers: Arc<RwLock<ControllerMap>>,
        active_controllers: Arc<RwLock<HashSet<u64>>>,
        cancellation_token: CancellationToken,
    ) -> Result<Self> {
        ratatui::init();
        let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        let ui_state = UIState {
            page: UIPage::Home,
            selection_state: ListState::default(),
            detected_controllers,
            active_controllers,
        };
        Ok(Self {
            terminal,
            ui_state,
            cancellation_token,
        })
    }

    pub fn render(&mut self) {
        let _ = self
            .terminal
            .draw(|frame| Self::render_all(frame, &mut self.ui_state));
    }
    // Renders all components of the UI
    fn render_all(frame: &mut Frame, ui_state: &mut UIState) {
        match ui_state.page {
            UIPage::Home => Self::render_home(frame, ui_state),
            UIPage::Controllers => Self::render_controllers(frame, ui_state),
            _ => {}
        }
    }

    // Render the home page
    fn render_home(frame: &mut Frame, ui_state: &mut UIState) {
        let major_layout = Layout::horizontal([Constraint::Max(30)])
            .horizontal_margin(20)
            .flex(Flex::Center);
        let minor_layout = Layout::vertical([Constraint::Max(15)])
            .vertical_margin(10)
            .flex(Flex::Center);
        let [rect] = major_layout.areas(frame.area());
        let [rect] = minor_layout.areas(rect);
        let list = List::new(["Controllers", "Settings", "Exit"])
            .block(Block::bordered().title("Starboard"))
            .style(LAVENDER)
            .highlight_style(Style::default().bg(LAVENDER).fg(Color::Black));
        frame.render_stateful_widget(list, rect, &mut ui_state.selection_state);
    }

    fn render_controllers(frame: &mut Frame, ui_state: &mut UIState) {
        let detected_controllers = ui_state.detected_controllers.blocking_read();
        let active_controllers = ui_state.active_controllers.blocking_write();
        let detected_controller_names = detected_controllers
            .values()
            .map(|diagnostic| diagnostic.to_text());

        // The `None` values
        // being filtered out are ID's that
        // for some reason are present in
        // `active_controllers` but absent
        // from `detected_controllers`
        let active_controller_names = active_controllers
            .iter()
            .filter_map(|id| detected_controllers.get(id))
            .map(|diagnostic| diagnostic.name().to_text());

        let layout = Layout::horizontal([Constraint::Ratio(1, 2); 2]).horizontal_margin(5);
        let active_list = List::new(active_controller_names)
            .block(Block::bordered().title("Active Controllers"))
            .style(LAVENDER);
        let detected_list = List::new(detected_controller_names)
            .block(Block::bordered().title("Detected Controllers"))
            .style(LAVENDER)
            .highlight_style(Style::default().bg(LAVENDER).fg(Color::Black));

        let [active_rect, detected_rect] = layout.areas(frame.area());
        frame.render_widget(active_list, active_rect);
        frame.render_stateful_widget(detected_list, detected_rect, &mut ui_state.selection_state);
    }

    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(key) => self.handle_key_press(key),
            _ => {}
        }
    }

    fn handle_key_press(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('c') => {
                // TODO: Clean up this nested logic. Yes, I was lazy when I wrote it.
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.cancellation_token.cancel();
                }
            }
            _ => {}
        }
    }

    // Execute behavior based on the currently selected button
    fn on_enter(&mut self) {
        let selected = self.ui_state.selection_state.selected();
        let mut page = self.ui_state.page;
        match (page, selected) {
            (UIPage::Home, Some(0)) => page = UIPage::Controllers,
            (UIPage::Home, Some(1)) => page = UIPage::Settings,
            (UIPage::Home, Some(2)) => self.cancellation_token.cancel(),
            (UIPage::Controllers, Some(v)) => self.toggle_controller(v),
            _ => {}
        }
    }

    fn on_backspace(&mut self) {
        let mut page = self.ui_state.page;
        match page {
            UIPage::Controllers | UIPage::Settings => page = UIPage::Home,
            _ => {}
        }
    }

    // Toggles whether a detected controller is enabled or not
    fn toggle_controller(&self, selected: usize) {
        let detected_controllers = self.ui_state.detected_controllers.blocking_read();
        let mut active_controllers = self.ui_state.active_controllers.blocking_write();
        let controller = detected_controllers.values().nth(selected).unwrap();
        let id = controller.id();
        if active_controllers.contains(id) {
            active_controllers.remove(id);
        } else {
            active_controllers.insert(*id);
        }
    }
}

impl Drop for StarboardServerUI {
    fn drop(&mut self) {
        ratatui::restore();
    }
}
