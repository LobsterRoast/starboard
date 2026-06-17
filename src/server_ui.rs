use anyhow::Result;

// The struct to manage the UI that opens when the server application is opened
// The UI is created through `ratatui` and runs in a terminal
pub struct StarboardServerUI {}

impl StarboardServerUI {
    pub fn launch_ui(&self) -> Result<()> {
        ratatui::run(|mut terminal| {});
        Ok(())
    }
}
