use anyhow::Result;

// The struct to manage the UI that opens when the server application is opened
// The UI is created through `ratatui` and runs in a terminal
pub struct StarboardServerUI {}

impl StarboardServerUI {
    pub fn new() -> Self {
        Self {}
    }

    pub fn launch_ui(&self) -> Result<()> {
        let mut quit = false;
        ratatui::run(|mut terminal| -> Result<()> {
            Ok(loop {
                let _ = terminal.draw(|frame| frame.render_widget("Foo", frame.area()))?;
            })
        });
        Ok(())
    }
}
