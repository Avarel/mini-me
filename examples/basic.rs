use minime::{editor::keybindings::NormalKeybinding, editor::Editor, Result};

// Basic bare bones example.
//
// Sample output:
//
//      Write something cool!
//      hello there
//      how are you?
//      [examples\basic.rs:14] term.read_multiline() = Ok(
//          "hello there\nhow are you?",
//      )

fn main() -> Result<()> {
    println!("Write something cool!");
    // Build the prompt.
    let term = Editor::default();

    dbg!(term.read(NormalKeybinding))?;
    Ok(())
}
