use crossterm::event::*;
use crossterm::event;
use std::cmp;

use crate::reader;
use crate::output;
use crate::prompt;

const QUIT_TIMES: u8 = 1;

pub enum MODE {
   Normal,
   Insert,
}

pub struct Editor {
    mode: MODE,
    reader: reader::Reader,
    output: output::Output,
    quit_times: u8,
}

impl Editor {
    pub fn new() -> Self {
        Self { 
            reader: reader::Reader,
            output: output::Output::new(),
            quit_times: QUIT_TIMES,
            mode: MODE::Normal,
        }
    }

    pub fn process_keypress(&mut self) -> crossterm::Result<bool> {
        match self.reader.read_key()? {
            KeyEvent {
                code: KeyCode::Char('w'),
                modifiers: event::KeyModifiers::CONTROL,
            } => {
                if self.output.dirty > 0 && self.quit_times > 0 {
                    self.output.status_message.set_message(format!(
                    "WARNING! File has unsaved changes. Press Ctrl-W {} more time to quit.",
                    self.quit_times
                ));
                self.quit_times -= 1;
                return Ok(true);
                }
                return Ok(false);
            }

            KeyEvent {
                code: KeyCode::Char(val @ ('h' | 'j' | 'k' | 'l')),
                modifiers: KeyModifiers::NONE,
            } => { 
                    if matches!(self.mode, MODE::Normal) {
                        self.output.move_cursor(val);
                    }
                    else {
                        self.output.insert_char(val);
                    }
            }

            KeyEvent {
                code: KeyCode::Char(val @ 'e'),
                modifiers: KeyModifiers::CONTROL,
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
            } => {
                if matches!(val, KeyCode::PageUp) {
                    self.output.cursor_controller.cursor_y = 
                        self.output.cursor_controller.row_offset
                } else {
                    self.output.cursor_controller.cursor_y = cmp::min(
                        self.output.win_size.1 + self.output.cursor_controller.row_offset -1,
                        self.output.editor_rows.number_of_rows(),
                    );
                }

                (0..self.output.win_size.1).for_each(|_| {
                    self.output.move_cursor_arrows(if matches!(val, KeyCode::PageUp) {
                        KeyCode::Up    
                } else {
                    KeyCode::Down
                });
            })
        }
            
            KeyEvent {
                code: KeyCode::Char('s'),
                modifiers: KeyModifiers::CONTROL,
            } =>  {
                if matches!(self.output.editor_rows.filename, None) {
                    let prompt = prompt!(&mut self.output, "Save as : {} (ESC to cancel)")
                        .map(|it| it.into());
                    if let None = prompt {
                        self.output
                            .status_message
                            .set_message("Save Aborted".into());
                        return Ok(true);
                    }
                    self.output.editor_rows.filename = prompt
                }
                self.output.editor_rows.save().map(|len| {
                    self.output
                        .status_message
                        .set_message(format!("{} bytes written to disk", len));
                    self.output.dirty = 0
                })?;
            }

            KeyEvent { 
                code: KeyCode::Char('f'), 
                modifiers: KeyModifiers::CONTROL, 
            } => {
                self.output.find()?;
            }

            KeyEvent { 
                code: key @ (KeyCode::Backspace | KeyCode::Delete), 
                modifiers: KeyModifiers::NONE,
            } => {
                if matches!(key, KeyCode::Delete) {
                    self.output.move_cursor_arrows(KeyCode::Right)
                }
                self.output.delete_char();
            }   

            KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
            } => {
                    if matches!(self.mode, MODE::Insert) {
                        self.output.inser_newline();
                    }
                }

            KeyEvent { 
                code: KeyCode::Char('i'), 
                modifiers: KeyModifiers::NONE, 
            } => {  
                    if matches!(self.mode, MODE::Normal) {
                        self.mode = MODE::Insert;
                        self.output.status_message.set_message(format!("INSERT"));
                    }
                    else {
                        self.output.insert_char('i');
                    }
                }

            KeyEvent { 
                code: KeyCode::Char('a'), 
                modifiers: KeyModifiers::NONE, 
            } => {
                    if matches!(self.mode, MODE::Normal) {
                        self.mode = MODE::Insert;
                        self.output.status_message.set_message(format!("INSERT"));
                    }
                    else {
                        self.output.insert_char('a');
                    }
                }

            KeyEvent {
                code: code @ (KeyCode::Char(..) | KeyCode::Tab),
                modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
            } => {
                    if matches!(self.mode, MODE::Insert) {
                        self.output.insert_char(match code {
                            KeyCode::Tab => '\t',
                            KeyCode::Char(ch) => ch,
                            _ => unreachable!(),
                        });
                    }
                }
                
            KeyEvent { 
                code: KeyCode::Esc, 
                modifiers: KeyModifiers::NONE, 
            } => { 
                    self.mode = MODE::Normal;
                    self.output.status_message.set_message(format!("NORMAL"));
                }

            _ => {}
        }
        Ok(true)
    }

    pub fn run(&mut self) -> crossterm::Result<bool> {
        self.output.refresh_screen()?;
        self.process_keypress()
    }
}
