use multiline_console::renderer::{lazy::LazyRenderer};
use multiline_console::crossterm::Result;

// Basic bare bones example.
//
// Sample output:
//      
//      Write something cool!
//      hello there
//      how are you?
//      [examples\basic.rs:14] term.read_multiline() = Ok(
//          "hello there\nhow are you?",
//      )

fn main() -> Result<()> {
    println!("Write something cool!");
    let term = multiline_console::Editor::builder()
        // The lazy renderer is much more efficient.
        .renderer(LazyRenderer::default())
        // Build the prompt.
        .build();

    dbg!(term.read_multiline())?;
    Ok(())
}