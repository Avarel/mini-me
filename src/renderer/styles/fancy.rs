use super::Style;
use crate::{renderer::Editor, Result};
use crossterm::{
    style::Colorize,
    terminal::{Clear, ClearType},
    QueueableCommand,
};
use std::io::Write;

pub struct FancyStyle<'s> {
    header_message: Option<&'s str>,
    gutter_message: &'s str,
}

impl<'s> FancyStyle<'s> {
    pub fn new() -> Self {
        Self {
            header_message: None,
            gutter_message: ""
        }
    }

    pub fn with_gutter_message(self, gutter_message: &'s str) -> Self {
        Self {
            gutter_message,
            ..self
        }
    }

    pub fn with_header_message(self, header_message: Option<&'s str>) -> Self {
        Self {
            header_message,
            ..self
        }
    }
}

impl<W: Write> Style<W> for FancyStyle<'_> {
    fn footer_rows(&self) -> usize {
        1
    }

    fn header_rows(&self) -> usize {
        if self.header_message.is_some() {
            1
        } else {
            0
        }
    }

    fn gutter_width(&self) -> usize {
        9
    }

    fn draw_header(&mut self, write: &mut W, _: &Editor) -> Result<()> {
        if let Some(header_message) = self.header_message {
            write!(
                write,
                "{} {}",
                "       ".black().on_dark_grey(),
                header_message
            )?;
            write.queue(Clear(ClearType::UntilNewLine))?;
        }
        Ok(())
    }

    fn draw_gutter(&mut self, write: &mut W, line_idx: usize, data: &Editor) -> Result<()> {
        if line_idx + 1 > data.line_count() {
            write!(write, "{}  ", "       ".on_dark_grey())?;
        } else if line_idx + 1 == data.line_count() && data.line(line_idx).len() == 0 {
            if line_idx == data.selection.focus.ln as usize {
                write!(
                    write,
                    "{} {}",
                    "      ▶ ".black().on_green(),
                    self.gutter_message.dark_grey()
                )?;
            } else {
                write!(
                    write,
                    "{}  {}",
                    "     ▶ ".black().on_green(),
                    self.gutter_message.dark_grey()
                )?;
            }
        } else if line_idx == data.selection.focus.ln as usize {
            write!(
                write,
                "{} ",
                format!("  {:>5} ", line_idx + 1).black().on_dark_grey()
            )?;
        } else {
            write!(
                write,
                "{}  ",
                format!(" {:>5} ", line_idx + 1).black().on_dark_grey()
            )?;
        }

        Ok(())
    }

    fn draw_footer(&mut self, write: &mut W, data: &Editor) -> Result<()> {
        write!(
            write,
            "{}{}{}{}",
            "  info ".black().on_dark_grey(),
            format!(" Lines: {:>3} ", data.line_count()),
            format!(" Chars: {:>3} ", data.char_count()),
            format!(
                " Ln {}, Col {} ",
                data.selection.focus.ln,
                data.selection.focus.col.min(data.curr_ln().len())
            )
        )?;

        write.queue(Clear(ClearType::UntilNewLine))?;
        Ok(())
    }
}
