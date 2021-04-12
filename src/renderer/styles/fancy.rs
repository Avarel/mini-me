use super::{Footer, Header, Margin};
use crate::{renderer::RenderData, Result};
use crossterm::{
    style::Colorize,
    terminal::{Clear, ClearType},
    QueueableCommand,
};
use std::io::Write;

pub struct FancyHeader<'s> {
    pub message: &'s str
}

impl<W: Write> Header<W> for FancyHeader<'_> {
    fn rows(&self) -> usize {
        1
    }

    fn draw(&mut self, w: &mut W, _: &RenderData) -> Result<()> {
        write!(
            w,
            "{} {}",
            "       ".black().on_dark_grey(),
            self.message
        )?;
        w.queue(Clear(ClearType::UntilNewLine))?;
        Ok(())
    }
}

pub struct FancyGutter;

impl FancyGutter {
    const MSG: &'static str = "Press enter to submit";
}

impl<W: Write> Margin<W> for FancyGutter {
    fn width(&self) -> usize {
        9
    }

    fn draw(&mut self, write: &mut W, line_idx: usize, data: &RenderData) -> Result<()> {
        if line_idx + 1 == data.line_count() && data.last_line().len() == 0 {
            if line_idx == data.focus().ln as usize {
                write!(
                    write,
                    "{} {}",
                    "      ▶ ".black().on_green(),
                    Self::MSG.dark_grey()
                )?;
            } else {
                write!(
                    write,
                    "{}  {}",
                    "     ▶ ".black().on_green(),
                    Self::MSG.dark_grey()
                )?;
            }
        } else if line_idx == data.focus().ln as usize {
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
}

pub struct FancyFooter;

impl<W: Write> Footer<W> for FancyFooter {
    fn rows(&self) -> usize {
        1
    }

    fn draw(&mut self, w: &mut W, data: &RenderData) -> Result<()> {
        write!(
            w,
            "{}{}{}{}",
            "  info ".black().on_dark_grey(),
            format!(" Lines: {:>3} ", data.line_count()),
            format!(" Chars: {:>3} ", data.char_count()),
            format!(
                " Ln {}, Col {} ",
                data.focus().ln,
                data.focus().col.min(data.current_line().len())
            )
        )?;

        w.queue(Clear(ClearType::UntilNewLine))?;
        Ok(())
    }
}
