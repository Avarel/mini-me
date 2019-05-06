use console::Term;
use std::io;

pub fn move_cursor_right(out: &Term, n: usize) -> io::Result<()> {
    if n > 0 {
        out.write_str(&format!("\x1b[{}C", n))
    } else {
        Ok(())
    }
}

pub fn move_cursor_left(out: &Term, n: usize) -> io::Result<()> {
    if n > 0 {
        out.write_str(&format!("\x1b[{}D", n))
    } else {
        Ok(())
    }
}