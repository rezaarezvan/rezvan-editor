use crossterm::terminal;

pub mod editorcontents;
pub mod editorrows;
pub mod reader;
pub mod cursorcontroller;
pub mod output;
pub mod cleanup;
pub mod editor;

fn main() -> crossterm::Result<()> {
    let _clean_up = cleanup::CleanUp;
    terminal::enable_raw_mode()?;
    let mut editor = editor::Editor::new();
    while editor.run()? {}
    Ok(())
}
