use minime::{
    editor::{keybindings::NormalKeybinding, Editor},
    renderer::{
        full::CrosstermRenderer,
        styles::classic::{ClassicFooter, ClassicGutter, ClassicHeader},
        styles::StyleBundle,
    },
    Result,
};

fn main() -> Result<()> {
    // Redirect our output to stdout (default).
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();

    let style = StyleBundle::new()
        .max_height(Some(10))
        .margin(ClassicGutter)
        .header(ClassicHeader {
            message: "Enter on the last line or Esc to submit your input!",
        })
        .footer(ClassicFooter);

    let renderer = CrosstermRenderer::render_to(&mut lock).style(style);

    // Print out some prompt using styling options.
    let mut term = Editor::default();
    term.read(NormalKeybinding, renderer)?;
    dbg!(term.contents());
    Ok(())
}
