use std::io;
use multiline_console::console::style;
use multiline_console::renderer::{LazyRenderer, FullRenderer};

// Example where there is a simple gutter on the side.
//
// Sample output:
//      
//       >>>  | Write something cool!
//          1 | hello there
//          2 | how are you?
//      [examples\gutter.rs:26] term.read_multiline() = Ok(
//          "hello there\nhow are you?",
//      )

fn main() -> io::Result<()> {
    println!("        Write something cool!");
    let term = multiline_console::MultilineTerm::builder()
        // Create a lazy renderer (more efficient) that prints out the gutter on the side.
        .renderer(LazyRenderer::wrap(FullRenderer::with_gutter(move |i, term| {
            // Signal that you're supposed to ENTER when the buffer is
            // empty/has a length of zero in order to submit the response.
            if term.buffers().is_empty() || i + 1 == term.buffers().len() && term.buffers().last().unwrap().len() == 0 {
                format!("{} ", style(" enter ").on_green())
            } else {
                format!("{} ", style(format!(" {:>5} ", i + 1)).on_blue())
            }
        })))
        // Build the prompt.
        .build_stdout();

    dbg!(term.read_multiline())?;
    Ok(())
}