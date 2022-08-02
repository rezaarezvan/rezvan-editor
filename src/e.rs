use crossterm::event::*;
use crossterm::event;

use crate::reader;
use crate::op;

pub struct Editor {
    reader: reader::Reader,
    output: op::Output,
}

impl Editor {
    pub fn new() -> Self {
        Self { 
            reader: reader::Reader,
            output: op::Output::new(),
        }
    }

    pub fn process_keypress(&mut self) -> crossterm::Result<bool> {
        match self.reader.read_key()? {
            KeyEvent {
                code: KeyCode::Char('w'),
                modifiers: event::KeyModifiers::CONTROL,
            } => return Ok(false),

            KeyEvent {
                code: KeyCode::Char(val @ ('w' | 'a' | 's' | 'd')),
                modifiers: KeyModifiers::NONE,
            } => self.output.move_cursor(val),

            KeyEvent {
                code: direction @ 
                (KeyCode::Up 
                | KeyCode::Down 
                | KeyCode::Left 
                | KeyCode::Right
                | KeyCode::Home
                | KeyCode::End),
                modifiers: KeyModifiers::NONE,
            } => self.output.move_cursor_arrows(direction),

            KeyEvent {
                code: val @ (KeyCode::PageUp | KeyCode::PageDown),
                modifiers: KeyModifiers::NONE,
            } => (0..self.output.win_size.1).for_each(|_| {
                self.output.move_cursor_arrows(if matches!(val, KeyCode::PageUp) {
                    KeyCode::Up    
                } else {
                    KeyCode::Down
                });
            }),

            _ => {}
        }
        Ok(true)
    }

    pub fn run(&mut self) -> crossterm::Result<bool> {
        self.output.refresh_screen()?;
        self.process_keypress()
    }
}