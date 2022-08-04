use crossterm::style::*;
use crossterm::queue;

use crate::row;
use crate::editorcontents;


pub enum HighlightType {
    Normal,
    Number,
}

pub trait SyntaxHighlight {
    fn syntax_color(&self, highlight_type: &HighlightType) -> Color;

    fn update_syntax(&self, at: usize, editor_rows: &mut Vec<row::Row>);

    fn color_row(&self, render: &str, highlight: &[HighlightType], out: &mut editorcontents::EditorContents) {
        render.chars().enumerate().for_each(|(i, c)| {
            let _ = queue!(out, SetForegroundColor(self.syntax_color(&highlight[i])));
            out.push(c);
            let _ = queue!(out, ResetColor);
        });
    }
}

#[macro_export]
macro_rules! syntax_struct {
    (
        struct $Name:ident;
    ) => {
        struct $Name;
        
        impl SyntaxHighlight for $Name {

            fn syntax_color(&self, highlight_type: &HighlightType) -> Color {
                match highlight_type {
                    HightlightType::Normal => Color::Reset,
                    HightlightType::Number => Color::Cyan,
                }
            }

            fn update_syntax(&self, at: usize, editor_rows: &mut Vec<row::Row>) {
                let current_row = &mut editor_rows[at];
                macro_rules! add {
                    ($h:expr) => {
                        current_row.highlight.push($h)
                    };
                }

                current_row.highlight = Vec::with_capacity(current_row.render.len());
                let chars = current_row.render.chars();
                for c in chars {
                    if c.is_digit(10) {
                        add!(HighlightType::Number);
                    } 
                    add!(HighlightType::Normal)
                    
                }
                assert_eq!(current_row.render.len(), current_row.highlight.len())
            }
        }
    };
}