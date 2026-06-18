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
    selection_state: UISelectionState,
}

impl StarboardServerUI {
    pub fn new() -> Self {
        Self {
            running: false,
            page: UIPage::Home,
            selection_state: UISelectionState::Tabs { index: 0 },
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
    fn render(&self, frame: &mut Frame) {
        let layout = Layout::vertical([Constraint::Max(1), Constraint::Max(100)])
            .flex(Flex::Legacy)
            .horizontal_margin(15)
            .vertical_margin(3);
        let [tabs_area, content_area] = layout.areas(frame.area());
        self.render_home(frame);
    }

    // Render the home page
    fn render_home(&self, frame: &mut Frame) {
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
        frame.render_stateful_widget(list, rect, &mut ListState::default().with_selected(Some(0)));
    }

    // Render tabs to select either the `Controllers` or `Settings` menus
    fn render_tabs(&self, frame: &mut Frame, rect: Rect) {
        let tabs = Tabs::new(["Controllers", "Settings"])
            .select(match self.selection_state {
                UISelectionState::Tabs { index } => Some(index),
                _ => None,
            })
            .style(LAVENDER);
        frame.render_widget(tabs, rect);
    }

    fn render_content_box(&self, frame: &mut Frame, rect: Rect) {
        let block = Block::new().style(LAVENDER);
        let layout = Layout::horizontal([Constraint::Ratio(1, 2); 2]);

        frame.render_widget(block, rect);

        self.render_controller_lists(frame, layout.areas(rect));
    }

    fn render_controller_lists(&self, frame: &mut Frame, rects: [Rect; 2]) {
        let active_list = List::new(["Controller 1", "Controller 2"])
            .block(Block::bordered().title("Active Controllers"));
        let detected_list = List::new(["Controller 1", "Controller 2", "Controller 3"])
            .block(Block::bordered().title("Detected Controllers"));

        frame.render_widget(active_list, rects[0]);
        frame.render_widget(detected_list, rects[1]);
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
            KeyCode::Left => self.selection_state = self.selection_state.left(),
            KeyCode::Right => self.selection_state = self.selection_state.right(),
            KeyCode::Up => self.selection_state = self.selection_state.up(),
            KeyCode::Down => self.selection_state = self.selection_state.down(),
            _ => {}
        }
        Ok(())
    }
}
