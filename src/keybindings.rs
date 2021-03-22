use crate::{editor::Editor, renderer::Renderer};

use crossterm::{
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
    Result,
};

pub trait Keybinding {
    fn read(&self, editor: &mut Editor<impl Renderer>) -> Result<bool>;
}

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
    fn clamp_cursor_index(editor: &mut Editor<impl Renderer>) {
        editor.cursor.col = editor.cursor.col.min(editor.current_line_len());
    }

    fn process_key_event(editor: &mut Editor<impl Renderer>, event: KeyEvent) -> Result<bool> {
        let code = event.code;
        match code {
            KeyCode::Down => {
                if event.modifiers.contains(KeyModifiers::SHIFT) {
                    editor.cursor.ln = editor.line_count() - 1;
                } else if editor.cursor.ln + 1 < editor.line_count() {
                    editor.cursor.ln += 1;
                }
            }
            KeyCode::Up => {
                if event.modifiers.contains(KeyModifiers::SHIFT) {
                    editor.cursor.ln = 0;
                } else if editor.cursor.ln > 0 {
                    editor.cursor.ln -= 1;
                }
            }
            KeyCode::Left => {
                Self::clamp_cursor_index(editor);
                if editor.cursor.col > 0 {
                    editor.cursor.col -= 1;
                } else if editor.cursor.ln > 0 {
                    // Move to the end of the previous line.
                    editor.cursor.ln -= 1;
                    editor.cursor.col = editor.current_line_len();
                }
            }
            KeyCode::Right => {
                Self::clamp_cursor_index(editor);
                let len = editor.current_line_len();
                if editor.cursor.col < len {
                    editor.cursor.col += 1;
                } else if editor.cursor.ln + 1 < editor.line_count() {
                    // Move to the beginning of the next line.
                    editor.cursor.ln += 1;
                    editor.cursor.col = 0;
                }
            }
            KeyCode::Backspace => {
                Self::clamp_cursor_index(editor);
                if editor.cursor.col > 0 {
                    editor.cursor.col = editor.delete_char_at_cursor(-1);
                } else if editor.cursor.ln > 0 {
                    // Backspace at the beginning of the line, so push the contents of
                    // the current line to the line above it, and remove the line.

                    // Push the content of the current line to the previous line.
                    let cbuf = editor.remove_line(editor.cursor.ln);
                    // Change line number.
                    editor.cursor.ln -= 1;

                    // The cursor should now be at the end of the previous line
                    // before appending the contents of the current line.
                    editor.cursor.col = editor.current_line_len();
                    editor.push_line_str(editor.cursor.ln, &cbuf);
                }
            }
            KeyCode::Delete => {
                Self::clamp_cursor_index(editor);
                if editor.cursor.col < editor.current_line_len() {
                    editor.cursor.col = editor.delete_char_at_cursor(0);
                } else if editor.cursor.ln + 1 < editor.line_count() {
                    // Push the content of the next line to the this line.
                    let cbuf = editor.remove_line(editor.cursor.ln + 1);
                    editor.push_line_str(editor.cursor.ln, &cbuf);
                }
            }
            KeyCode::Tab => {
                Self::clamp_cursor_index(editor);
                let soft = 4 - editor.cursor.col % 4;
                for _ in 0..soft {
                    editor.cursor.col = editor.insert_char_at_cursor(0, ' ');
                }
            }
            KeyCode::Char(c) => {
                Self::clamp_cursor_index(editor);
                editor.cursor.col = editor.insert_char_at_cursor(0, c);
            }
            KeyCode::Esc => {
                // Quick escape and finish the input.

                // Move to the end if cursor is not on last line.
                if editor.cursor.ln + 1 != editor.line_count() || editor.current_line_len() != 0 {
                    editor.cursor.ln = editor.line_count();
                    editor.insert_line(editor.line_count(), "");
                }

                return Ok(false);
            }
            KeyCode::Enter => {
                if editor.cursor.ln + 1 == editor.line_count() && editor.current_line_len() == 0 {
                    // Enter on the last line of the prompt which is also empty
                    // finishes the input.
                    return Ok(false);
                }

                Self::clamp_cursor_index(editor);

                // Split the input after the cursor.
                let cursor_idx = editor.cursor.col;
                editor.split_line(editor.cursor.ln, cursor_idx);

                editor.cursor.col = 0;
                editor.cursor.ln += 1;
            }
            _ => { /* ignore */ }
        }
        Ok(true)
    }
}
