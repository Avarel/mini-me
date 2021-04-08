use crate::{editor::Editor, renderer::Renderer, Result};

use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyModifiers};

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
    fn process_key_event(editor: &mut Editor<impl Renderer>, event: KeyEvent) -> Result<bool> {
        let code = event.code;
        let ln_count = editor.line_count();
        let mut cursor = editor.cursor();
        // let shifted = event.modifiers.contains(KeyModifiers::SHIFT);
        let alt = event.modifiers.contains(KeyModifiers::ALT);
        let control = event.modifiers.contains(KeyModifiers::CONTROL);
        match code {
            // KeyCode::Down if shifted => cursor.move_to_bottom(),
            // KeyCode::Up if control => cursor.move_to_top(),
            // KeyCode::PageDown => cursor.move_to_bottom(),
            // KeyCode::PageUp => cursor.move_to_top(),

            KeyCode::Down => cursor.move_down(),
            KeyCode::Up => cursor.move_up(),
            KeyCode::Left => cursor.move_left(),
            KeyCode::Right => cursor.move_right(),
            
            KeyCode::Home => {
                let leading_spaces = cursor
                    .curr_ln()
                    .chars()
                    .take_while(|c| c.is_whitespace())
                    .count();
                if cursor.col() == leading_spaces {
                    cursor.move_to_col(0);
                } else {
                    cursor.move_to_col(leading_spaces);
                }
            }
            KeyCode::End => cursor.move_to_line_end(),

            KeyCode::Backspace => cursor.backspace(),
            KeyCode::Char('h') if control => cursor.backspace(),
            KeyCode::Delete => cursor.delete(),

            KeyCode::Tab => {
                cursor.clamp();
                let soft = 4 - cursor.col() % 4;
                for _ in 0..soft {
                    cursor.insert_char(0, ' ');
                }
                *cursor.col_mut() += soft;
            }
            KeyCode::BackTab => {
                cursor.clamp();
                let col = cursor.col();
                cursor.move_to_col(0);
                let mut count = 0;
                for _ in 0..4 {
                    if cursor.curr_char() == ' ' {
                        cursor.delete();
                        count += 1;
                    } else {
                        break;
                    }
                }
                cursor.move_to_col(col - count);
            }
            KeyCode::Esc => return Ok(false),
            KeyCode::Enter if !alt && cursor.curr_ln_len() == 0 && cursor.ln() + 1 == ln_count => {
                return Ok(false)
            }
            KeyCode::Enter => cursor.type_char('\n'),
            KeyCode::Char(c) => cursor.type_char(c),
            _ => { /* ignored */ },
        }
        Ok(true)
    }
}

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
        let mut cursor = editor.cursor();
        match code {
            KeyCode::Esc => return Ok(false),
            _ => cursor.insert_str(&format!("{:#?}", event)),
        }
        Ok(true)
    }
}
