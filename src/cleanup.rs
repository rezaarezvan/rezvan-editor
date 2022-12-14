use crate::output;
use crossterm::terminal;

pub struct CleanUp;

impl Drop for CleanUp {
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("Unable to disable raw mode");
        output::Output::clear_screen().expect("Error");
    }
}
