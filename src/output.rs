use crossterm::event::*;
use crossterm::terminal::ClearType;
use crossterm::{cursor, execute, queue, terminal, style};

use std::io::stdout;
use std::io::Write;
use std::io;
use std::cmp;

use crate::editorrows;
use crate::editorcontents;
use crate::cursorcontroller;
use crate::status;
use crate::output;
use crate::reader;
use crate::searchindex;
use crate::searchdirection;

const VERSION: f32 = 0.1;

pub struct Output {
    pub win_size: (usize, usize),
    pub editor_contents: editorcontents::EditorContents,
    pub cursor_controller: cursorcontroller::CursorController,
    pub editor_rows: editorrows::EditorRows,
    pub status_message: status::StatusMessage,
    pub dirty: u64,
    pub search_index: searchindex::SearchIndex,
}

#[macro_export]
macro_rules! prompt {
    /* modify */
    ($output:expr,$args:tt) => {
        prompt!($output, $args, callback = |&_, _, _| {})
    };
    ($output:expr,$args:tt, callback = $callback:expr) => {{
        let output: &mut output::Output = $output;
        let mut input = String::with_capacity(32);
        loop {
            output.status_message.set_message(format!($args, input)); // modify
            output.refresh_screen()?;
            let key_event = reader::Reader.read_key()?;
            match key_event {
                KeyEvent {
                    code: KeyCode::Enter,
                    modifiers: KeyModifiers::NONE,
                } => {
                    if !input.is_empty() {
                        output.status_message.set_message(String::new());
                        $callback(output, &input, KeyCode::Enter); // add line
                        break;
                    }
                }
                KeyEvent {
                    code: KeyCode::Esc, ..
                } => {
                    output.status_message.set_message(String::new());
                    input.clear();
                    $callback(output, &input, KeyCode::Esc); // add line
                    break;
                }
                KeyEvent {
                    code: KeyCode::Backspace | KeyCode::Delete,
                    modifiers: KeyModifiers::NONE,
                } => {
                    input.pop();
                }
                KeyEvent {
                    code: code @ (KeyCode::Char(..) | KeyCode::Tab),
                    modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                } => {
                    input.push(match code {
                        KeyCode::Tab => '\t',
                        KeyCode::Char(ch) => ch,
                        _ => unreachable!(),
                    });
                }
                _ => {}
            }
            $callback(output, &input, key_event.code); // add line
        }
        if input.is_empty() {
            None
        } else {
            Some(input)
        }
    }};
}

impl Output {
    pub fn new() -> Self {
        let win_size = terminal::size()
            .map(|(x, y)| (x as usize, y as usize - 2))
            .unwrap();
        Self { 
            win_size,
            editor_contents: editorcontents::EditorContents::new(),
            cursor_controller: cursorcontroller::CursorController::new(win_size),
            editor_rows: editorrows::EditorRows::new(),
            status_message: status::StatusMessage::new("HELP: Ctrl-S = Save | Ctrl-W = Quit | Ctrl-F Find".into()),
            dirty: 0,
            search_index: searchindex::SearchIndex::new(),
        }
    }


    pub fn find(&mut self) -> io::Result<()> {
        let cursor_controller = self.cursor_controller;
        if prompt!(
            self,
            "Search: {} (Use ESC / Arrows / Enter)",
            callback = Output::find_callback
        ).is_none() {
            self.cursor_controller = cursor_controller;
        }
        Ok(())
    }

    pub fn find_callback(output: &mut Output, keyword: &str, key_code: KeyCode) {
        match key_code {
            KeyCode::Esc | KeyCode::Enter => {
                output.search_index.reset();
            }
            _ => {
                output.search_index.y_direction = None;
                match key_code {
                    KeyCode::Down => {
                        output.search_index.y_direction = searchdirection::SearchDirection::Forward.into()
                    }
                    KeyCode::Up => {
                        output.search_index.y_direction = searchdirection::SearchDirection::Backward.into()
                    }
                    _ => {}
                }
                for i in 0..output.editor_rows.number_of_rows() {
                    let row_index = match output.search_index.y_direction.as_ref() {
                        None => {
                            output.search_index.y_index = i;
                            i
                        }
                        Some(dir) => {
                            if matches!(dir, searchdirection::SearchDirection::Forward) {
                                output.search_index.y_index + i + 1
                            } else {
                                let res = output.search_index.y_index.saturating_sub(i);
                                if res == 0 {
                                    break;
                                }
                                res - 1
                            }
                        }
                    };
                    if row_index > output.editor_rows.number_of_rows() - 1 {
                        break;
                    }
                    let row = output.editor_rows.get_editor_row(row_index);
                    let index = match output.search_index.x_direction.as_ref() {
                        None => row.render.find(&keyword),
                        Some(dir) => {
                            let index = if matches!(dir, searchdirection::SearchDirection::Forward) {
                                let start =
                                    cmp::min(row.render.len(), output.search_index.x_index + 1);
                                row.render[start..]
                                    .find(&keyword)
                                    .map(|index| index + start)
                            } else {
                                row.render[..output.search_index.x_index].rfind(&keyword)
                            };
                            if index.is_none() {
                                break;
                            }
                            index
                        }
                    };
                    if let Some(index) = index {
                        output.cursor_controller.cursor_y = row_index;
                        output.search_index.y_index = row_index;
                        output.search_index.x_index = index;
                        output.cursor_controller.cursor_x = row.get_row_content_x(index);
                        output.cursor_controller.row_offset = output.editor_rows.number_of_rows();
                        break;
                    }
                }
            }
        }
    }

    pub fn draw_message_bar(&mut self) {
        queue!(
            self.editor_contents,
            terminal::Clear(ClearType::UntilNewLine)
        )
        .unwrap();
        if let Some(msg) = self.status_message.message() {
            self.editor_contents
                .push_str(&msg[..cmp::min(self.win_size.0, msg.len())]);
        }
    }

    pub fn draw_status_bar(&mut self) {
        self.editor_contents
            .push_str(&style::Attribute::Reverse.to_string());
        let info = format!(
            "{} {} -- {} lines",
            self.editor_rows
                .filename
                .as_ref()
                .and_then(|path| path.file_name())
                .and_then(|name| name.to_str())
                .unwrap_or("[No Name]"),
            if self.dirty > 0 { "(modified)" } else { "" },
            self.editor_rows.number_of_rows()
        );
        let info_len = cmp::min(info.len(), self.win_size.0);
        let line_info = format!(
            "{}/{}",
            self.cursor_controller.cursor_y + 1,
            self.editor_rows.number_of_rows()
        );
        self.editor_contents.push_str(&info[..info_len]);
        for i in info_len .. self.win_size.0 {
            if self.win_size.0 - i == line_info.len() {
                self.editor_contents.push_str(&line_info);
                break;
            } else {
                self.editor_contents.push(' ');
            }
        }  

        self.editor_contents
            .push_str(&style::Attribute::Reset.to_string());
        self.editor_contents.push_str("\r\n");
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
                let row = self.editor_rows.get_render(file_row);
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
            //if i < screen_rows - 1 {
                self.editor_contents.push_str("\r\n");
            //}
        }
    }

    pub fn refresh_screen(&mut self) -> crossterm::Result<()> {
        self.cursor_controller.scroll(&self.editor_rows);
        queue!(self.editor_contents, cursor::Hide, cursor::MoveTo(0, 0))?;
        self.draw_rows();
        self.draw_status_bar();
        self.draw_message_bar(); // add line
        let cursor_x = self.cursor_controller.render_x - self.cursor_controller.column_offset;
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

    pub fn insert_char(&mut self, ch: char) {
        if self.cursor_controller.cursor_y == self.editor_rows.number_of_rows() {
            self.editor_rows.insert_row(self.editor_rows.number_of_rows(), String::new());
            self.dirty += 1;
        }
        self.editor_rows
            .get_editor_row_mut(self.cursor_controller.cursor_y)
            .insert_char(self.cursor_controller.cursor_x, ch);
        self .cursor_controller.cursor_x += 1;
        self.dirty += 1;
    }

    pub fn inser_newline(&mut self) {
        if self.cursor_controller.cursor_x == 0 {
            self.editor_rows
                .insert_row(self.cursor_controller.cursor_y, String::new())
        } else {
            let current_row = self
                .editor_rows
                .get_editor_row_mut(self.cursor_controller.cursor_y);
            let new_row_content = current_row.row_content[self.cursor_controller.cursor_x..].into();
            current_row
                .row_content
                .truncate(self.cursor_controller.cursor_x);
            editorrows::EditorRows::render_row(current_row);
            self.editor_rows
                .insert_row(self.cursor_controller.cursor_y + 1, new_row_content);
        }
    }

    pub fn delete_char(&mut self) {
        if self.cursor_controller.cursor_y == self.editor_rows.number_of_rows() {
            return;
        }
        if self.cursor_controller.cursor_y == 0 && self.cursor_controller.cursor_x == 0 {
            return;
        }
        let row = self
            .editor_rows
            .get_editor_row_mut(self.cursor_controller.cursor_y);
        if self.cursor_controller.cursor_x > 0 {
            row.delete_char(self.cursor_controller.cursor_x - 1);
            self.cursor_controller.cursor_x -= 1;
        }
        else {
            let previous_row_content = self
                .editor_rows
                .get_row(self.cursor_controller.cursor_y - 1);
            self.cursor_controller.cursor_x = previous_row_content.len();
            self.editor_rows
                .join_adjacent_rows(self.cursor_controller.cursor_y);
            self.cursor_controller.cursor_y -= 1;
        }
        self.dirty += 1;
    }
}
