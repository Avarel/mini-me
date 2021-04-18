use super::{Style};
use crate::{renderer::Editor, Result};
use crossterm::{
    style::Colorize,
    terminal::{Clear, ClearType},
    QueueableCommand,
};
use std::io::Write;

// pub struct FancyHeader<'s> {
//     pub message: &'s str
// }

pub struct FancyStyle<'s> {
    pub message: &'s str
}


impl<W: Write> Style<W> for FancyStyle<'_> {
    fn footer_rows(&self) -> usize {
        1
    }

    fn header_rows(&self) -> usize {
        todo!()
    }

    fn gutter_width(&self) -> usize {
        todo!()
    }

    fn draw_header(&mut self, write: &mut W, data: &Editor) -> Result<()> {
        todo!()
    }

    fn draw_gutter(&mut self, write: &mut W, line_idx: usize, data: &Editor) -> Result<()> {
        if line_idx + 1 > data.line_count() {
            write!(
                write,
                "{}  ",
                "       ".on_dark_grey()
            )?;
        } else if line_idx + 1 == data.line_count() && data.line(line_idx).len() == 0 {
            if line_idx == data.selection.focus.ln as usize {
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
            "{} {}",
            "       ".black().on_dark_grey(),
            self.message
        )?;
        write.queue(Clear(ClearType::UntilNewLine))?;
        Ok(())
    }
}

// impl Header for FancyHeader<'_> {
//     fn rows(&self) -> usize {
//         1
//     }

//     fn draw(&mut self, w: &mut dyn Write, _: &Editor) -> Result<()> {
//         write!(
//             w,
//             "{} {}",
//             "       ".black().on_dark_grey(),
//             self.message
//         )?;
//         w.queue(Clear(ClearType::UntilNewLine))?;
//         Ok(())
//     }
// }

// pub struct FancyGutter;

// impl FancyGutter {
//     const MSG: &'static str = "Press enter to submit";
// }

// impl Margin for FancyGutter {
//     fn width(&self) -> usize {
//         9
//     }

//     fn draw(&mut self, write: &mut dyn Write, line_idx: usize, data: &Editor) -> Result<()> {
//         if line_idx + 1 > data.line_count() {
//             write!(
//                 write,
//                 "{}  ",
//                 "       ".on_dark_grey()
//             )?;
//         } else if line_idx + 1 == data.line_count() && data.line(line_idx).len() == 0 {
//             if line_idx == data.selection.focus.ln as usize {
//                 write!(
//                     write,
//                     "{} {}",
//                     "      ▶ ".black().on_green(),
//                     Self::MSG.dark_grey()
//                 )?;
//             } else {
//                 write!(
//                     write,
//                     "{}  {}",
//                     "     ▶ ".black().on_green(),
//                     Self::MSG.dark_grey()
//                 )?;
//             }
//         } else if line_idx == data.selection.focus.ln as usize {
//             write!(
//                 write,
//                 "{} ",
//                 format!("  {:>5} ", line_idx + 1).black().on_dark_grey()
//             )?;
//         } else {
//             write!(
//                 write,
//                 "{}  ",
//                 format!(" {:>5} ", line_idx + 1).black().on_dark_grey()
//             )?;
//         }

//         Ok(())
//     }
// }

// pub struct FancyFooter;

// impl Footer for FancyFooter {
//     fn rows(&self) -> usize {
//         1
//     }

//     fn draw(&mut self, w: &mut dyn Write, data: &Editor) -> Result<()> {
//         write!(
//             w,
//             "{}{}{}{}",
//             "  info ".black().on_dark_grey(),
//             format!(" Lines: {:>3} ", data.line_count()),
//             format!(" Chars: {:>3} ", data.char_count()),
//             format!(
//                 " Ln {}, Col {} ",
//                 data.selection.focus.ln,
//                 data.selection.focus.col.min(data.curr_ln().len())
//             )
//         )?;

//         w.queue(Clear(ClearType::UntilNewLine))?;
//         Ok(())
//     }
// }
