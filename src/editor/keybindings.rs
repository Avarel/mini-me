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

fn shifted(event: KeyEvent) -> bool {
    event.modifiers.contains(KeyModifiers::SHIFT)
}

impl NormalKeybinding {
    fn process_key_event(editor: &mut Editor<impl Renderer>, event: KeyEvent) -> Result<bool> {
        let code = event.code;
        let ln_count = editor.line_count();
        let mut cursor = editor.cursor();
        match code {
            KeyCode::Char(c) => cursor.type_char(c),
            KeyCode::Down if shifted(event) => cursor.move_to_bottom(),
            KeyCode::Down => cursor.move_down(),
            KeyCode::Up if shifted(event) => cursor.move_to_top(),
            KeyCode::Up => cursor.move_up(),
            KeyCode::Left if shifted(event) => cursor.move_to_col(0),
            KeyCode::Left => cursor.move_left(),
            KeyCode::Right if shifted(event) => cursor.move_to_col(cursor.current_line_len()),
            KeyCode::Right => cursor.move_right(),
            KeyCode::Backspace => cursor.backspace(),
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
            KeyCode::Enter if cursor.current_line_len() == 0 && cursor.ln() + 1 == ln_count => {
                return Ok(false)
            }
            KeyCode::Enter => {
                cursor.type_char('\n');
            }
            _ => { /* ignore */ }
        }
        Ok(true)
    }
}
