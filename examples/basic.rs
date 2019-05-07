use std::io;

fn main() -> io::Result<()> {
    let prompt = multiline_console::MultilineTerm::builder()
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