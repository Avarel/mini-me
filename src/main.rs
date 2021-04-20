use std::{
    fs::OpenOptions,
    io::{BufReader, BufWriter, Write},
};

#[cfg(feature = "bin")]
use clap::{App, Arg};

use minime::{
    editor::{keybindings::NormalKeybinding, Editor},
    renderer::{full::CrosstermRenderer, styles::fancy::FancyStyle},
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
                .help("Do not save the editor content to any location")
                .long("no-save")
                .short("ns")
                .conflicts_with("OUTPUT"),
        )
        .arg(
            Arg::with_name("NOEMIT")
                .help("Do not emit the editor content if a file is not specified")
                .long("no-emit")
                .short("ne")
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
    let no_emit = matches.is_present("NOEMIT");

    let output_path = if no_save {
        None
    } else {
        matches.value_of("OUTPUT").or(file_path)
    };

    let file = file_path.and_then(|path| OpenOptions::new().read(true).open(path).ok());

    let stderr = std::io::stderr();
    let mut lock = BufWriter::new(stderr.lock());

    let style = FancyStyle::new()
        .with_header_message(file_path)
        .with_gutter_message("Press enter to submit");

    let renderer = CrosstermRenderer::render_to(&mut lock)
        .max_height(max_height)
        .with_style(style);

    let mut term = Editor::default();

    if let Some(file) = file {
        term.set_contents(BufReader::new(file))?;
        term.move_to_bottom();
        term.move_to_line_end(false);
    }

    term.read(NormalKeybinding, renderer)?;

    let contents = term.contents();

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
    } else if !no_emit {
        let stdout = std::io::stdout();
        let mut writer = stdout.lock();
        writer.write_all(contents.as_bytes())?;
        writer.write(b"\n")?;
    }

    Ok(())
}
