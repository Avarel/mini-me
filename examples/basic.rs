use std::io;
use multiline_console::console::style;

fn main() -> io::Result<()> {
    println!("{} Write something cool!", style(" >>> ").black().on_green());
    let term = multiline_console::MultilineTerm::builder()
        // Print out the gutter.
        .initial_buffers(vec!["hey".to_owned(), "there".to_owned()])
        .gutter(move |i, term| {
            // Signal that you're supposed to ENTER when the buffer is
            // empty/has a length of zero in order to submit the response.
            if term.buffers().is_empty() || i + 1 == term.buffers().len() && term.buffers().last().unwrap().len() == 0 {
                format!("enter | ")
            } else {
                format!("{:>5} | ", i + 1)
            }
        })
        // Build the prompt.
        .build_stdout();

    dbg!(term.read_multiline())?;
    Ok(())
}