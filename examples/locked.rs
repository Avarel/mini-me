use in_edit::{crossterm::Result, keybindings::NormalKeybinding};
use in_edit::{editor::Editor, renderer::full::CrosstermRenderer};

fn main() -> Result<()> {
    // Redirect our output to stdout (default).
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    // Print out some prompt using styling options.
    let term = Editor::with_renderer(CrosstermRenderer::render_to(&mut lock));
    println!("{}", term.read(NormalKeybinding)?);
    Ok(())
}
