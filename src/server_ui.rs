use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{Terminal, backend::Backend};
use std::time::Duration;

// Keeps track of what the user currently has selected in a UI
// TODO: Add more enumerable values based on which widgets are used in the UI
enum UISelectionState {
    Tabs { index: u8 },
}

// The struct to manage the UI that opens when the server application is opened
// The UI is created through `ratatui` and runs in a terminal
pub struct StarboardServerUI {
    selection_state: UISelectionState,
}

impl StarboardServerUI {
    pub fn new() -> Self {
        Self {
            selection_state: UISelectionState::Tabs { index: 0 },
        }
    }

    pub fn launch_ui(&self) -> Result<()> {
        ratatui::run(ui_loop)
    }
}

// The main loop that the UI will call every frame
fn ui_loop(terminal: &mut ratatui::DefaultTerminal) -> Result<()> {
    while (!check_sigint()?) {
        let _ = terminal.draw(|frame| frame.render_widget("Foo", frame.area()))?;
    }
    Ok(())
}

// Returns whether or not Ctrl+C is pressed
fn check_sigint() -> Result<bool> {
    if !event::poll(Duration::from_millis(16))? {
        Ok(false)
    } else if let Event::Key(key) = event::read()? {
        Ok(key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
    } else {
        Ok(false)
    }
}
