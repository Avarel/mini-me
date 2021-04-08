# Mini-Me
An embeddable customizable multiline text-editor based on `crossterm`.
```rust
      ╭─── Input Prompt ─────────
    1 │ hello there
    2 │ this is a simple prompt
    3 │ thats multiline and decent enough
    4 ┃ _
      ╰─── Lines: 4     Chars: 70    Ln: 3, Col: 0
```

## Features
* Simple, intuitive, and embeddable.
* Customize header, footer, and or margin gutters.

## Binary Installation
`minime` can be used as a CLI. The best way to install it is using:
```bash
cargo install --features=bin --path .
```
The binary can be used by invoking `minime -h`.

## Default Controls
* Arrow keys work as expected.
* Home, end, delete, Tab and backtab mirrors VSCode behavior.
* `Esc` or `Enter` on the last empty line to close and submit the prompt.

## Usage

### Basic Setup
This is the most basic setup available.
```rust
use minime::{editor::keybindings::NormalKeybinding, editor::Editor, Result};

fn main() -> Result<()> {
    println!("Write something cool!");

    let term = Editor::default();

    dbg!(term.read(NormalKeybinding))?;
    Ok(())
}
```

### Advanced
You can lock `stdout()` or `stderr()` to get better performance. You can also
customize several settings in the renderer.
```rust
use minime::{
    editor::{keybindings::NormalKeybinding, Editor},
    renderer::{
        full::CrosstermRenderer,
        styles::classic::{ClassicFooter, ClassicGutter, ClassicHeader},
    },
    Result,
};

fn main() -> Result<()> {
    // Redirect our output to stdout (default).
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();

    let renderer = CrosstermRenderer::render_to(&mut lock)
        .max_height(Some(10))
        .margin(ClassicGutter)
        .header(ClassicHeader(
            "Enter on the last line or Esc to submit your input!",
        ))
        .footer(ClassicFooter);

    // Print out some prompt using styling options.
    let term = Editor::with_renderer(renderer);
    dbg!(term.read(NormalKeybinding)?);
    Ok(())
}
```