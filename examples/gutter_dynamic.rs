use std::io::{self, Write};

use crossterm::style::SetBackgroundColor;
use in_edit::crossterm::{
    style::{self, style, Color, Colorize, Colors, ResetColor, SetColors},
    Result,
};
use in_edit::{
    editor::Editor,
    renderer::{data::RenderData, full::FullRenderer},
};

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

// fn gutter_formatter(data: &RenderData, i: usize, write: &mut dyn Write) -> io::Result<()> {
//     // Signal that you're supposed to ENTER when the buffer is
//     // empty/has a length of zero in order to submit the response.
//     if data.line_count() == 0 {
//         write!(write, "{} ", " enter ".black().on_green())
//     } else if i + 1 == data.line_count() && data.last_line().len() == 0 {
//         if i == data.cursor().line as usize {
//             write!(write, "{} ", "  enter ".black().on_green())?;
//             data.write_line(i, write)
//         } else {
//             write!(write, "{}  ", " enter ".black().on_green())?;
//             data.write_line(i, write)
//         }
//     } else if i == data.cursor().line as usize {
//         // The full renderer allows for a live updating gutter chevron that follows the cursor.
//         write!(
//             write,
//             "{}  {:>5} {}{} ",
//             SetColors(Colors::new(Color::Black, Color::Grey)),
//             i + 1,
//             ResetColor,
//             SetBackgroundColor(style::Color::DarkGrey)
//         )?;
//         data.write_line(i, write)
//     } else {
//         write!(
//             write,
//             "{} {:>5} {}  ",
//             SetColors(Colors::new(Color::Black, Color::Grey)),
//             i + 1,
//             ResetColor
//         )?;
//         data.write_line(i, write)
//     }
// }

fn main() -> Result<()> {
    // Redirect our output to stdout (default).
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    // Print out some prompt using styling options.
    println!("{}  Write something cool!", "  >>>  ".black().on_green());
    let term = Editor::builder()
        // Render to our stdout.
        // Create a renderer with a formatter that allows for a gutter on the side.
        // .renderer(FullRenderer::render_with_formatter_to(
        //     &mut lock,
        //     &gutter_formatter,
        // )) // No lazy renderer since the prompt gutter must update globally
        .renderer(FullRenderer::render_to(&mut lock))
        // Build the prompt.
        .build();

    dbg!(term.read_multiline())?;
    Ok(())
}
