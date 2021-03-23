use in_edit::{
    crossterm::Result,
    editor::{keybindings::NormalKeybinding, Editor},
    renderer::{full::CrosstermRenderer, margin::SimpleGutter},
};

fn main() -> Result<()> {
    // Redirect our output to stdout (default).
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    // Print out some prompt using styling options.
    let term = Editor::with_renderer(CrosstermRenderer::render_to_with(
        &mut lock,
        SimpleGutter::new(5),
    ));
    dbg!(term.read(NormalKeybinding)?);
    Ok(())
}
