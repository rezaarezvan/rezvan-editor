use crossterm::event::*;
use std::{cmp};
use crate::editorrows;

pub struct CursorController {
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub screen_columns: usize,
    pub screen_rows: usize,
    pub row_offset: usize,
    pub column_offset: usize,
}

impl CursorController {

    pub fn new(win_size: (usize, usize)) -> CursorController {
        Self {
            cursor_x: 0,
            cursor_y: 0,
            screen_columns: win_size.0,
            screen_rows: win_size.1,
            row_offset: 0,
            column_offset:0,
        }
    }

    pub fn move_cursor(&mut self, direction: char, editor_rows: &editorrows::EditorRows) {
        let number_of_rows = editor_rows.number_of_rows();
        match direction {
            'w' => {
                self.cursor_y = self.cursor_y.saturating_sub(1);
            }


            'a' => {
                if self.cursor_x != 0 {
                    self.cursor_x -= 1;
                }
            }

            's' => {
                if self.cursor_y < number_of_rows {
                    self.cursor_y += 1;
                }
            }

            'd' => {
                if self.cursor_y < number_of_rows
                    && self.cursor_x < editor_rows.get_row(self.cursor_y).len() 
                {
                    self.cursor_x += 1;
                }
            }

            _ => unimplemented!(),
        }
    }

    pub fn move_cursor_arrows(&mut self, direction: KeyCode, editor_rows: &editorrows::EditorRows) {
        let number_of_rows = editor_rows.number_of_rows();
        match direction {
            KeyCode::Up => {
                self.cursor_y =  self.cursor_y.saturating_sub(1);
            }
            KeyCode::Left => {
                if self.cursor_x != 0 {
                    self.cursor_x -= 1;
                }
            }
            KeyCode::Down => {
                if self.cursor_y < number_of_rows{
                    self.cursor_y += 1;
                }
            }
            KeyCode::Right => {
                if self.cursor_y < number_of_rows
                    && self.cursor_x < editor_rows.get_row(self.cursor_y).len() 
                {
                    self.cursor_x += 1;
                }
            }

            KeyCode::End => self.cursor_x = self.screen_columns - 1,
            KeyCode::Home => self.cursor_x = 0,
            _ => unimplemented!(),
        }
    }

    pub fn scroll(&mut self) {
        self.row_offset = cmp::min(self.row_offset, self.cursor_y);
        if self.cursor_y >= self.row_offset + self.screen_rows {
            self.row_offset = self.cursor_y - self.screen_rows + 1;
        }
        
        self.column_offset = cmp::min(self.column_offset, self.cursor_x);
        if self.cursor_x >= self.column_offset + self.screen_columns {
            self.column_offset = self.cursor_x - self.screen_columns + 1;
        }
    }

}

pub fn main() {
    
}