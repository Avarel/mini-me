use mini_me::{crossterm::Result, editor::{keybindings::NormalKeybinding, Editor}, renderer::{footer::ClassicFooter, full::CrosstermRenderer, header::ClassicHeader, margin::GutterMargin}};

fn main() -> Result<()> {
    // Redirect our output to stdout (default).
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();

    // let m: Box<dyn Margin<_>> = Box::new(GutterMargin::new(5));
    // let renderer = CrosstermRenderer::render_to(&mut lock).margin(m);

    let renderer = CrosstermRenderer::render_to(&mut lock)
        .margin(GutterMargin::new(5))
        .header(ClassicHeader)
        .footer(ClassicFooter);

    // Print out some prompt using styling options.
    let term = Editor::with_renderer(renderer);
    dbg!(term.read(NormalKeybinding)?);
    Ok(())
}
