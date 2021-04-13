use std::{
    fs::OpenOptions,
    io::{BufReader, BufWriter, Write},
};

#[cfg(feature = "bin")]
use clap::{App, Arg};

use minime::{
    editor::{keybindings::NormalKeybinding, Editor},
    renderer::{
        full::CrosstermRenderer,
        styles::fancy::{FancyFooter, FancyGutter},
    },
    Result,
};

const CARGO_PKG_VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() -> Result<()> {
    let matches = App::new("Mini-Me")
        .version(CARGO_PKG_VERSION)
        .author("Avarel <avarelpm@gmail.com>")
        .about("Miniaturized text editor")
        .arg(
            Arg::with_name("HEIGHT")
                .long("size")
                .short("s")
                .value_name("INTEGER")
                .help("Sets the window height of the editor")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("FILE")
                .help("Sets the input file to use")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("NOSAVE")
                .long("nosave")
                .short("ns")
                .conflicts_with("OUTPUT"),
        )
        .arg(
            Arg::with_name("OUTPUT")
                .short("o")
                .long("output")
                .help("Sets the file to save to")
                .default_value("FILE")
                .takes_value(true),
        )
        .get_matches();

    let max_height = matches.value_of("HEIGHT").and_then(|s| s.parse().ok());

    let file_path = matches.value_of("FILE");

    let no_save = matches.is_present("NOSAVE");

    let output_path = if no_save {
        None
    } else {
        matches.value_of("OUTPUT").or(file_path)
    };

    let file = file_path.and_then(|path| OpenOptions::new().read(true).open(path).ok());

    let stderr = std::io::stderr();
    let mut lock = BufWriter::new(stderr.lock());

    let renderer = CrosstermRenderer::render_to(&mut lock)
        .max_height(max_height)
        .margin(FancyGutter)
        .footer(FancyFooter);

    let mut term = Editor::with_renderer(renderer);

    if let Some(file) = file {
        term.set_contents(BufReader::new(file))?;
        term.move_to_bottom();
        term.move_to_line_end(false);
    }

    let contents = term.read(NormalKeybinding)?;

    if let Some(file) = output_path.and_then(|path| {
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .ok()
    }) {
        let mut writer = BufWriter::new(file);
        writer.write_all(contents.as_bytes())?;
    } else {
        let stdout = std::io::stdout();
        let mut writer = stdout.lock();
        writer.write_all(contents.as_bytes())?;
        writer.write(b"\n")?;
    }

    Ok(())
}
