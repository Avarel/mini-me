use minime::{
    editor::{keybindings::DebugKeybinding, Editor},
    renderer::{
        full::CrosstermRenderer,
        styles::fancy::{FancyFooter, FancyGutter, FancyHeader},
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
        .header(FancyHeader {
            message: "Type something :)",
        })
        .margin(FancyGutter)
        .footer(FancyFooter);

    let renderer = CrosstermRenderer::render_to(&mut lock)
        .style(style);

    // Print out some prompt using styling options.
    let mut term = Editor::default();

    term.read(DebugKeybinding, renderer)?;
    dbg!(term.contents());
    Ok(())
}
