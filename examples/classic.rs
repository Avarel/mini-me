use minime::{
    editor::{keybindings::NormalKeybinding, Editor},
    renderer::{
        full::CrosstermRenderer,
        styles::classic::{ClassicFooter, ClassicGutter, ClassicHeader},
    },
    Result,
};

fn main() -> Result<()> {
    // Redirect our output to stdout (default).
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();

    let renderer = CrosstermRenderer::render_to(&mut lock)
        .max_height(Some(10))
        .margin(ClassicGutter)
        .header(ClassicHeader {
            message: "Enter on the last line or Esc to submit your input!",
        })
        .footer(ClassicFooter);

    // Print out some prompt using styling options.
    let mut term = Editor::default();
    term.read(NormalKeybinding, renderer)?;
    dbg!(term.contents());
    Ok(())
}
