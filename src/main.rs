mod console_patch;
mod term;

use std::io;
use term::*;

fn main() -> io::Result<()> {
    let prompt = MultilineTerm::builder()
        .prompt(|i, size, line| {
            if i + 1 == size && line.len() == 0 {
                " > | ".to_owned()
            } else {
                format!("{:^3}| ", i + 1)
            }
        })
        .build_stdout();

    let result = prompt.read_multiline()?;
    dbg!(result);
    Ok(())
}
