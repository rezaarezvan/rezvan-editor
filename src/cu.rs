use crossterm::terminal;
use crate::op;

pub struct CleanUp;

impl Drop for CleanUp {
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("Unable to disable raw mode");
        op::Output::clear_screen().expect("Error");
    }
}