use crate::Editor;

use crossterm::{
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
    // terminal::{disable_raw_mode, enable_raw_mode},
    Result,
};

pub trait Keybinding {
    fn read(&self, editor: &mut Editor) -> Result<bool>;
}

pub struct NormalKeybinding;

impl Keybinding for NormalKeybinding {
    fn read(&self, editor: &mut Editor) -> Result<bool> { 
        let key_event = read()?;
        match key_event {
            Event::Key(k) => {
                Self::process_key_event(editor, k)
            }
            _ => Ok(true)
        }
    }
}

impl NormalKeybinding {
    fn process_key_event(editor: &mut Editor, event: KeyEvent) -> Result<bool> {
        let code = event.code;
        match code {
            KeyCode::Down => {
                if event.modifiers.contains(KeyModifiers::SHIFT) {
                    editor.cursor.line = if editor.buffers.len() == 0 {
                        0
                    } else {
                        editor.buffers.len() - 1
                    };
                } else if editor.buffers.is_empty() {
                    return Ok(true);
                } else if editor.cursor.line + 1 < editor.buffers.len() {
                    editor.cursor.line += 1;
                }
            }
            KeyCode::Up => {
                if event.modifiers.contains(KeyModifiers::SHIFT) {
                    editor.cursor.line = 0;
                } else if editor.buffers.is_empty() {
                    return Ok(true);
                } else if editor.cursor.line > 0 {
                    editor.cursor.line -= 1;
                }
            }
            KeyCode::Left => {
                if editor.buffers.is_empty() {
                    return Ok(true);
                }
                editor.cursor.index = editor.clamp_cursor_index();
                if editor.cursor.index > 0 {
                    editor.cursor.index -= 1;
                } else if editor.cursor.line > 0 {
                    // Move to the end of the previous line.
                    editor.cursor.line -= 1;
                    editor.cursor.index = editor.buffers[editor.cursor.line].len();
                }
            }
            KeyCode::Right => {
                if editor.buffers.is_empty() {
                    return Ok(true);
                }
                editor.cursor.index = editor.clamp_cursor_index();
                let len = editor.current_line().len();
                if editor.cursor.index < len {
                    editor.cursor.index += 1;
                } else if editor.cursor.line + 1 < editor.buffers.len() {
                    // Move to the beginning of the next line.
                    editor.cursor.line += 1;
                    editor.cursor.index = 0;
                }
            }
            KeyCode::Backspace => {
                if editor.buffers.is_empty() {
                    return Ok(true);
                }
                editor.cursor.index = editor.clamp_cursor_index();

                if editor.cursor.index > 0 {
                    editor.cursor.index = editor.delete_char_before_cursor();
                } else if editor.cursor.line > 0 {
                    // Backspace at the beginning of the line, so push the contents of
                    // the current line to the line above it, and remove the line.

                    // Push the content of the current line to the previous line.
                    let cbuf = editor.buffers.remove(editor.cursor.line);
                    // Change line number.
                    editor.cursor.line -= 1;

                    // The cursor should now be at the end of the previous line
                    // before appending the contents of the current line.
                    editor.cursor.index = editor.current_line().len();
                    editor.current_line_mut().push_str(&cbuf);
                }
            }
            KeyCode::Delete => {
                if editor.buffers.is_empty() {
                    return Ok(true);
                }
                editor.cursor.index = editor.clamp_cursor_index();

                if editor.cursor.index < editor.current_line().len() {
                    editor.cursor.index = editor.delete_char_after_cursor();
                } else if editor.cursor.line + 1 < editor.buffers.len() {
                    // Push the content of the next line to the this line.
                    let cbuf = editor.buffers.remove(editor.cursor.line + 1);
                    editor.current_line_mut().push_str(&cbuf);
                }
            }
            KeyCode::Tab => {
                editor.cursor.index = editor.clamp_cursor_index();
                let soft = 4 - editor.current_line().len() % 4;
                for _ in 0..soft {
                    editor.cursor.index = editor.insert_char_before_cursor(' ');
                }
            }
            KeyCode::Char(c) => {
                editor.cursor.index = editor.clamp_cursor_index();
                editor.cursor.index = editor.insert_char_before_cursor(c);
            }
            KeyCode::Esc => {
                // Quick escape and finish the input.

                // Move to the end if cursor is not on last line.
                if editor.cursor.line + 1 != editor.buffers.len() || editor.current_line().len() != 0 {
                    editor.cursor.line = if editor.buffers.len() == 0 {
                        0
                    } else {
                        editor.buffers.len()
                    };
                }
                
                editor.buffers.push(String::new());
                return Ok(false);
            }
            KeyCode::Enter => {
                if editor.buffers.len() == 0 {
                    return Ok(false);
                } else if editor.cursor.line + 1 == editor.buffers.len()
                    && editor.current_line().len() == 0
                {
                    // Enter on the last line of the prompt which is also empty
                    // finishes the input.

                    // Remove last useless line.
                    editor.buffers.remove(editor.cursor.line);
                    return Ok(false);
                } else {
                    editor.cursor.index = editor.clamp_cursor_index();
                    // Split the input after the cursor.
                    let cursor_idx = editor.cursor.index;
                    let cbuf = editor.current_line_mut();
                    let nbuf = cbuf.split_off(cursor_idx);

                    // Create a new line and move the cursor to the next line.
                    editor.buffers.insert(editor.cursor.line + 1, nbuf);
                    editor.cursor.index = 0;
                    editor.cursor.line += 1;
                }
            }
            _ => { /* ignore */ }
        }
        Ok(true)
    }
}