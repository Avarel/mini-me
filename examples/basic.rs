use std::io;
use multiline_console::console::style;

fn main() -> io::Result<()> {
    let prompt = multiline_console::MultilineTerm::builder()
        // Anchor the last line on the bottom
        .anchor(multiline_console::Anchor::Bottom) 
        // Print out the prompt.
        .prompt(move |i, term| {
            // Signal that you're supposed to ENTER when the buffer is
            // empty/has a length of zero in order to submit the response.
            if term.buffers().is_empty() || i + 1 == term.buffers().len() && term.buffers().last().unwrap().len() == 0 {
                format!("{} ", style(" enter ").black().on_green())
            } else {
                format!("{} ", style(format!(" {:>5} ", i + 1)).black().on_blue())
            }
        })
        // Build the prompt.
        .build_stdout();

    dbg!(prompt.read_multiline())?;
    Ok(())
}