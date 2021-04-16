use minime::{
    editor::{keybindings::NormalKeybinding, Editor},
    renderer::full::CrosstermRenderer,
    Result,
};

fn main() -> Result<()> {
    // Redirect our output to stdout (default).
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();

    let renderer = CrosstermRenderer::render_to(&mut lock);

    // Print out some prompt using styling options.
    let mut term = Editor::default();
    dbg!(term.read(NormalKeybinding, renderer)?);
    Ok(())
}
