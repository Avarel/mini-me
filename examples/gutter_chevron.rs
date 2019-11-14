use std::io;
use multiline_console::console::style;
use multiline_console::renderer::FullRenderer;

// Example where there is a chevron in the gutter that follows the where the cursor.
//
// Sample output:
//      
//       >>>  | Write something cool!
//          1 | hello there
//      >   2 | how are you?
//      [examples\gutter_chevron.rs:26] term.read_multiline() = Ok(
//          "hello there\nhow are you?",
//      )

fn main() -> io::Result<()> {
    println!("{} | Write something cool!", style(" >>> ").black().on_green());
    let term = multiline_console::MultilineTerm::builder()
        // Create a renderer with a gutter on the side.
        .renderer(FullRenderer::with_gutter(move |i, term| {
            // Signal that you're supposed to ENTER when the buffer is
            // empty/has a length of zero in order to submit the response.
            if term.buffers().is_empty() || i + 1 == term.buffers().len() && term.buffers().last().unwrap().len() == 0 {
                format!("enter | ")
            } else if i == term.cursor().line {
                // The full renderer allows for a live updating gutter chevron that follows the cursor.
                format!(">{:4} | ", i + 1)
            } else {
                format!("{:>5} | ", i + 1)
            }
        })) // No lazy renderer since the prompt gutter must update globally
        // Build the prompt.
        .build_stdout();

    dbg!(term.read_multiline())?;
    Ok(())
}