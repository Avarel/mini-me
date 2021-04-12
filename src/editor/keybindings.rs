use crate::{editor::Editor, renderer::Renderer, Result};

use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyModifiers};

/// Generic keybinding trait.
pub trait Keybinding {
    /// Read a key from the environment and act upon the editor.
    fn read(&self, editor: &mut Editor<impl Renderer>) -> Result<bool>;
}

/// Default keybindings for the editor.
pub struct NormalKeybinding;

impl Keybinding for NormalKeybinding {
    fn read(&self, editor: &mut Editor<impl Renderer>) -> Result<bool> {
        let key_event = read()?;
        match key_event {
            Event::Key(k) => Self::process_key_event(editor, k),
            _ => Ok(true),
        }
    }
}

impl NormalKeybinding {
    fn process_key_event(editor: &mut Editor<impl Renderer>, event: KeyEvent) -> Result<bool> {
        let code = event.code;
        let ln_count = editor.line_count();
        // let mut cursor = editor.cursor();
        let shifted = event.modifiers.contains(KeyModifiers::SHIFT);
        let alt = event.modifiers.contains(KeyModifiers::ALT);
        let control = event.modifiers.contains(KeyModifiers::CONTROL);

        match code {  
            // KeyCode::Down if shifted => editor.move_to_bottom(),
            // KeyCode::Up if control => editor.move_to_top(),
            // KeyCode::PageDown => editor.move_to_bottom(),
            // KeyCode::PageUp => editor.move_to_top(),
            KeyCode::Down => editor.move_down(shifted),
            KeyCode::Up => editor.move_up(shifted),
            KeyCode::Left => editor.move_left(shifted),
            KeyCode::Right => editor.move_right(shifted),

            KeyCode::Home => {
                let leading_spaces = editor
                    .curr_ln()
                    .chars()
                    .take_while(|c| c.is_whitespace())
                    .count();
                if editor.col() == leading_spaces {
                    editor.move_to_col(0, shifted);
                } else {
                    editor.move_to_col(leading_spaces, shifted);
                }
            }
            KeyCode::End => editor.move_to_line_end(shifted),

            KeyCode::Backspace => editor.backspace(),
            KeyCode::Char('h') if control => editor.backspace(),
            KeyCode::Delete => editor.delete(),

            KeyCode::Tab => {
                editor.clamp();
                let soft = 4 - editor.col() % 4;
                for _ in 0..soft {
                    editor.insert_char(0, ' ');
                }
                *editor.col_mut() += soft;
            }
            KeyCode::BackTab => {
                editor.clamp();
                let col = editor.col();
                editor.move_to_col(0, false);
                let mut count = 0;
                for _ in 0..4 {
                    if editor.curr_char() == ' ' {
                        editor.delete();
                        count += 1;
                    } else {
                        break;
                    }
                }
                editor.move_to_col(col - count, false);
            }
            KeyCode::Esc => return Ok(false),
            KeyCode::Enter => {
                if !alt && editor.curr_ln_len() == 0 && editor.ln() + 1 == ln_count {
                    return Ok(false);
                } else {
                    editor.type_char('\n');
                }
            }
            KeyCode::Char(c) => editor.type_char(c),
            _ => { /* ignored */ }
        }
        Ok(true)
    }
}

#[doc(hidden)]
pub struct DebugKeybinding;

impl Keybinding for DebugKeybinding {
    fn read(&self, editor: &mut Editor<impl Renderer>) -> Result<bool> {
        let key_event = read()?;
        match key_event {
            Event::Key(k) => Self::process_key_event(editor, k),
            _ => Ok(true),
        }
    }
}

impl DebugKeybinding {
    fn process_key_event(editor: &mut Editor<impl Renderer>, event: KeyEvent) -> Result<bool> {
        let code = event.code;
        match code {
            KeyCode::Esc => return Ok(false),
            _ => editor.insert_str(&format!("{:#?}", event)),
        }
        Ok(true)
    }
}
