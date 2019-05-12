use std::io;
use multiline_console::renderer::LazyRenderer;

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

fn main() -> io::Result<()> {
    println!("Write something cool!");
    let term = multiline_console::MultilineTerm::builder()
        // The lazy renderer is much more efficient.
        .renderer(LazyRenderer::default())
        // Build the prompt.
        .build_stdout();

    dbg!(term.read_multiline())?;
    Ok(())
}