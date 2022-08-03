use crossterm::event::*;
use crossterm::terminal::ClearType;
use crossterm::{cursor, execute, queue, terminal};
use std::io::stdout;
use std::io::Write;
use std::{cmp};
use crate::editorrows;
use crate::editorcontents;
use crate::cursorcontroller;

const VERSION: f32 = 0.1;

pub struct Output {
    pub win_size: (usize, usize),
    pub editor_contents: editorcontents::EditorContents,
    pub cursor_controller: cursorcontroller::CursorController,
    pub editor_rows: editorrows::EditorRows
}

impl Output {
    pub fn new() -> Self {
        let win_size = terminal::size()
            .map(|(x, y)| (x as usize, y as usize))
            .unwrap();
        Self { 
            win_size,
            editor_contents: editorcontents::EditorContents::new(),
            cursor_controller: cursorcontroller::CursorController::new(win_size),
            editor_rows: editorrows::EditorRows::new(),
        }
    }

    pub fn clear_screen() -> crossterm::Result<()> {
        execute!(stdout(), terminal::Clear(ClearType::All))?;
        execute!(stdout(), cursor::MoveTo(0, 0))
    }

    pub fn draw_rows(&mut self) {
        let screen_rows = self.win_size.1;
        let screen_columns = self.win_size.0;
        for i in 0..screen_rows {
            let file_row = i + self.cursor_controller.row_offset;
            if file_row >= self.editor_rows.number_of_rows() {
                if /*add this condition*/ self.editor_rows.number_of_rows() == 0 && i == screen_rows / 3 {
                    let mut welcome = format!("Rezvan Editor --- Version {}", VERSION);
                    if welcome.len() > screen_columns {
                        welcome.truncate(screen_columns)
                    }
                    let mut padding = (screen_columns - welcome.len()) / 2;
                    if padding != 0 {
                        self.editor_contents.push('~');
                        padding -= 1
                    }
                    (0..padding).for_each(|_| self.editor_contents.push(' '));
                    self.editor_contents.push_str(&welcome);
                } else {
                    self.editor_contents.push('~');
                }
            } else {
                let row = self.editor_rows.get_row(file_row);
                let column_offset = self.cursor_controller.column_offset;
                let len = cmp::min(row.len().saturating_sub(column_offset), screen_columns);
                let start = if len == 0 { 0 } else { column_offset };
                self.editor_contents
                    .push_str(&row[start..start + len])
            }
            queue!(
                self.editor_contents,
                terminal::Clear(ClearType::UntilNewLine)
            )
            .unwrap();
            if i < screen_rows - 1 {
                self.editor_contents.push_str("\r\n");
            }
        }
    }

    pub fn refresh_screen(&mut self) -> crossterm::Result<()> {
        self.cursor_controller.scroll();
        queue!(
            self.editor_contents,
            cursor::Hide,
            cursor::MoveTo(0, 0)
        )?;
        self.draw_rows();
        let cursor_x = self.cursor_controller.cursor_x - self.cursor_controller.column_offset;
        let cursor_y = self.cursor_controller.cursor_y - self.cursor_controller.row_offset;
        queue!(
            self.editor_contents, 
            cursor::MoveTo(cursor_x as u16, cursor_y as u16),
            cursor::Show
        )?;
        self.editor_contents.flush()
    }

    pub fn move_cursor(&mut self, direction:char) {
        self.cursor_controller
            .move_cursor(direction, &self.editor_rows);
    }

    pub fn move_cursor_arrows(&mut self, direction:KeyCode) {
        self.cursor_controller
            .move_cursor_arrows(direction, &self.editor_rows);
    }
}