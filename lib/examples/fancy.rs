use mini_me::{
    editor::{keybindings::NormalKeybinding, Editor},
    renderer::{
        full::CrosstermRenderer,
        styles::fancy::{FancyFooter, FancyGutter, FancyHeader},
    },
    Result,
};

fn main() -> Result<()> {
    // Redirect our output to stdout (default).
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();

    let renderer = CrosstermRenderer::render_to(&mut lock)
        .max_height(Some(10))
        .header(FancyHeader("Type something :)"))
        .margin(FancyGutter)
        .footer(FancyFooter);

    // Print out some prompt using styling options.
    let term = Editor::with_renderer(renderer);
    dbg!(term.read(NormalKeybinding)?);
    Ok(())
}
