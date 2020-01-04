use in_edit::{Editor, renderer::full::FullRenderer};
use in_edit::crossterm::{Result, style::{self, style, Colorize}};

// Example where there is a chevron in the gutter that follows the where the cursor.
//
// Sample output:
//      
//       >>>  |  Write something cool!
//          1 |  hello there
//          2  | how are you?
//      [examples\gutter_dynamic.rs:44] term.read_multiline() = Ok(
//          "hello there\nhow are you?",
//      )

fn main() -> Result<()> {
    // Redirect our output to stdout (default).
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    // Print out some prompt using styling options.
    println!("{}  Write something cool!", "  >>>  ".black().on_green());
    let term = Editor::builder()
        // Render to our stdout.
        // Create a renderer with a formatter that allows for a gutter on the side.
        .renderer(FullRenderer::render_with_formatter_to(&mut lock, &|i, term| {
            // Signal that you're supposed to ENTER when the buffer is
            // empty/has a length of zero in order to submit the response.
            if term.buffers.is_empty() {
                format!("{} ", " enter ".black().on_green())
            } else if i + 1 == term.buffers.len() && term.buffers.last().unwrap().len() == 0 {
                if i == term.cursor.line as usize {
                    format!("{} {}", style(format!("  enter ")).black().on_green(), term.line(i))
                } else {
                    format!("{}  {}", style(format!(" enter ")).black().on_green(), term.line(i))
                }
            } else if i == term.cursor.line as usize {
                // The full renderer allows for a live updating gutter chevron that follows the cursor.
                format!("{}{} {}", style(format!("  {:>5} ", i + 1)).black().on_grey(), style::SetBackgroundColor(style::Color::DarkGrey), term.line(i))
            } else {
                format!("{}  {}", style(format!(" {:>5} ", i + 1)).black().on_grey(), term.line(i))
            }
        })) // No lazy renderer since the prompt gutter must update globally
        // Build the prompt.
        .build();

    dbg!(term.read_multiline())?;
    Ok(())
}