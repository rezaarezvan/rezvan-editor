use crate::editorrows;

#[derive(Default)]
pub struct Row {
    pub row_content: String,
    pub render: String,
}

impl Row {
    pub fn new(row_content: String, render: String) -> Self {
        Self {
            row_content,
            render,
        }
    }

    pub fn insert_char(&mut self, at: usize, ch: char) {
        self.row_content.insert(at, ch);
        editorrows::EditorRows::render_row(self)
    }

    pub fn delete_char(&mut self, at: usize) {
        self.row_content.remove(at);
        editorrows::EditorRows::render_row(self)
    }

    pub fn get_row_content_x(&self, render_x: usize) -> usize {
        let mut current_render_x = 0;
        for (cursor_x, ch) in self.row_content.chars().enumerate() {
            if ch == '\t' {
                current_render_x += (4 - 1) - (current_render_x % 4);
            }
            current_render_x += 1;
            if current_render_x > render_x {
                return cursor_x;
            }
        }
        0
    }
}
