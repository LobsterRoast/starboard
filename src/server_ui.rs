use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{Frame, Terminal, backend::Backend, widgets::*};
use std::time::Duration;

// Keeps track of what the user currently has selected in a UI
// TODO: Add more enumerable values based on which widgets are used in the UI
enum UISelectionState {
    Tabs { index: usize },
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
        ratatui::run(|term| self.ui_loop(term))
    }

    // The main loop that the UI will call every frame
    fn ui_loop(&self, terminal: &mut ratatui::DefaultTerminal) -> Result<()> {
        while (!check_sigint()?) {
            let _ = terminal.draw(|frame| self.render_tabs(frame))?;
        }
        Ok(())
    }

    // Render tabs to select either the `Controllers` or `Settings` menus
    fn render_tabs(&self, frame: &mut Frame) {
        let tabs = Tabs::new(["Controllers", "Settings"]).select(match self.selection_state {
            UISelectionState::Tabs { index } => Some(index),
            _ => None,
        });
        frame.render_widget(tabs, frame.area());
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
            _ => Ok(()),
        }
    }
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
