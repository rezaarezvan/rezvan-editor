use crossterm::terminal;

pub mod ec;
pub mod er;
pub mod reader;
pub mod cc;
pub mod op;
pub mod cu;
pub mod e;

fn main() -> crossterm::Result<()> {
    let _clean_up = cu::CleanUp;
    terminal::enable_raw_mode()?;
    let mut editor = e::Editor::new();
    while editor.run()? {}
    Ok(())
}
