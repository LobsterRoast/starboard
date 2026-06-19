use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{Frame, Terminal, backend::Backend, layout::*, prelude::*, widgets::*};
use std::time::Duration;

const LAVENDER: Color = Color::Rgb(150, 100, 175);

// Keeps track of what the user currently has selected in a UI
// TODO: Add more enumerable values based on which widgets are used in the UI
#[derive(Copy, Clone)]
enum UISelectionState {
    Tabs {
        index: usize,
    },
    DeviceList {
        major_index: usize,
        minor_index: usize,
    },
}

// Keeps track of what page the UI is currently on
#[derive(PartialEq, Copy, Clone)]
enum UIPage {
    Home,
    Controllers,
    Settings,
}

impl UISelectionState {
    // Activated when the left arrow key is pressed
    pub fn left(self) -> Self {
        match self {
            UISelectionState::Tabs { .. } => UISelectionState::Tabs { index: 0 },
            UISelectionState::DeviceList { minor_index, .. } => UISelectionState::DeviceList {
                major_index: 0,
                minor_index,
            },
        }
    }

    // Activated when the right arrow key is pressed
    pub fn right(self) -> Self {
        match self {
            UISelectionState::Tabs { .. } => UISelectionState::Tabs { index: 1 },
            UISelectionState::DeviceList { minor_index, .. } => UISelectionState::DeviceList {
                major_index: 1,
                minor_index,
            },
            _ => self,
        }
    }

    // Activated when the up arrow key is pressed
    pub fn up(self) -> Self {
        match self {
            UISelectionState::DeviceList {
                major_index,
                minor_index,
            } => UISelectionState::DeviceList {
                major_index,
                minor_index: minor_index - 1,
            },
            _ => self,
        }
    }

    // Activated when the up arrow key is pressed
    pub fn down(self) -> Self {
        match self {
            UISelectionState::Tabs { .. } => UISelectionState::DeviceList {
                major_index: 0,
                minor_index: 0,
            },
            UISelectionState::DeviceList {
                major_index,
                minor_index,
            } => UISelectionState::DeviceList {
                major_index,
                minor_index: minor_index + 1,
            },
        }
    }
}

// The struct to manage the UI that opens when the server application is opened
// The UI is created through `ratatui` and runs in a terminal
pub struct StarboardServerUI {
    running: bool,
    page: UIPage,
    selected: ListState,
}

impl StarboardServerUI {
    pub fn new() -> Self {
        Self {
            running: false,
            page: UIPage::Home,
            selected: ListState::default().with_selected(Some(0)),
        }
    }

    pub fn launch_ui(&mut self) -> Result<()> {
        self.running = true;
        ratatui::run(|term| self.ui_loop(term))
    }

    // The main loop that the UI will call every frame
    fn ui_loop(&mut self, terminal: &mut ratatui::DefaultTerminal) -> Result<()> {
        while self.running {
            self.poll_events()?;
            let _ = terminal.draw(|frame| self.render(frame))?;
        }
        Ok(())
    }

    // Renders all components of the UI
    fn render(&mut self, frame: &mut Frame) {
        let layout = Layout::vertical([Constraint::Max(1), Constraint::Max(100)])
            .flex(Flex::Legacy)
            .horizontal_margin(15)
            .vertical_margin(3);
        let [tabs_area, content_area] = layout.areas(frame.area());
        match self.page {
            UIPage::Home => self.render_home(frame),
            UIPage::Controllers => self.render_controllers(frame),
            _ => {}
        }
    }

    // Render the home page
    fn render_home(&mut self, frame: &mut Frame) {
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
        frame.render_stateful_widget(list, rect, &mut self.selected);
    }

    fn render_controllers(&mut self, frame: &mut Frame) {
        let layout = Layout::horizontal([Constraint::Ratio(1, 2); 2]).horizontal_margin(5);
        let active_list = List::new(["Controller 1", "Controller 2"])
            .block(Block::bordered().title("Active Controllers"))
            .style(LAVENDER);
        let detected_list = List::new(["Controller 1", "Controller 2", "Controller 3"])
            .block(Block::bordered().title("Detected Controllers"))
            .style(LAVENDER);

        let [active_rect, detected_rect] = layout.areas(frame.area());
        frame.render_stateful_widget(active_list, active_rect, &mut self.selected);
        frame.render_stateful_widget(detected_list, detected_rect, &mut self.selected);
    }

    // Checks to see if theres any events and matches them if there is
    fn poll_events(&mut self) -> Result<()> {
        while event::poll(Duration::from_millis(16))? {
            let event = event::read()?;
            self.match_event(event)?;
        }
        Ok(())
    }

    // Polls events and handles them appropriately
    fn match_event(&mut self, event: Event) -> Result<()> {
        // TODO: Implement functions for each possible event pattern
        match event {
            Event::Key(key) => self.on_key_event(key),
            _ => Ok(()),
        }
    }

    // Handled `Key` events
    fn on_key_event(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('c') => self.running = !key.modifiers.contains(KeyModifiers::CONTROL),
            KeyCode::Up => self.selected.scroll_up_by(1),
            KeyCode::Down => self.selected.scroll_down_by(1),
            KeyCode::Enter => self.on_enter(),
            KeyCode::Backspace => self.on_backspace(),
            _ => {}
        }
        Ok(())
    }

    // Execute behavior based on the currently selected button
    fn on_enter(&mut self) {
        let selected = self.selected.selected();
        match (self.page, selected) {
            (UIPage::Home, Some(0)) => self.page = UIPage::Controllers,
            (UIPage::Home, Some(1)) => self.page = UIPage::Settings,
            (UIPage::Home, Some(2)) => self.running = false,
            _ => {}
        }
    }

    fn on_backspace(&mut self) {
        match self.page {
            UIPage::Controllers | UIPage::Settings => self.page = UIPage::Home,
            _ => {}
        }
    }
}
