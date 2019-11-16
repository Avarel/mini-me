use multiline_console::renderer::{lazy::LazyRenderer, full::FullRenderer};
use multiline_console::crossterm::{Result, style::{style, Colorize}};

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

fn main() -> Result<()> {
    // Print out a basic prompt.
    println!("{} Write something cool!", "  >>>  ".black().on_green());
    let term = multiline_console::MultilineTerm::builder()
        // The renderer defaults to stdout.
        // Create a lazy renderer (more efficient) that prints out the gutter on the side.
        .renderer(LazyRenderer::wrap(FullRenderer::render_with_formatter(&|i, term| {
            // Signal that you're supposed to ENTER when the buffer is
            // empty/has a length of zero in order to submit the response.
            if term.buffers.is_empty() {
                format!("{} ", " enter ".black().on_green())
            } else if i + 1 == term.buffers.len() && term.buffers.last().unwrap().len() == 0 {
                format!("{} {}", " enter ".black().on_green(), term.line(i))
            } else {
                format!("{} {}", style(format!(" {:>5} ", i + 1)).on_blue(), term.line(i))
            }
        })))
        // Build the prompt.
        .build_stdout();

    dbg!(term.read_multiline())?;
    Ok(())
}

// pub struct Z {
//     b: A
//     z: String,
// }

// pub struct A {

// }

// impl Z {
//     fn doz(&mut self) {
//         self.b.take(&self);
//     }
// }

// impl A {
//     fn take(&mut self, data: &Z) {
//         println!()
//     }
// }