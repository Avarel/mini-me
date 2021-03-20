use ropey::{Rope, RopeSlice};

pub trait RopeExt {
    fn line_trimmed(&self, line_idx: usize) -> RopeSlice<'_>;
    fn remove_line(&mut self, line_idx: usize) -> String;
    fn insert_line(&mut self, line_idx: usize, string: &str);
}

impl RopeExt for Rope {
    fn line_trimmed(&self, line_idx: usize) -> RopeSlice<'_> {
        let line = self.line(line_idx);
        let line_len = line.len_chars();
        if line_len == 0 {
            line
        } else if line.char(line_len - 1) == '\n' {
            line.slice(..line_len - 1)
        } else {
            line.slice(..line_len)
        }
    }

    fn remove_line(&mut self, line_idx: usize) -> String {
        let line_start = self.line_to_char(line_idx);
        let line_end = self.line_to_char(line_idx + 1);
        let rm = self.line_trimmed(line_idx).to_string();
        self.remove(line_start..line_end);
        return rm;
    }

    fn insert_line(&mut self, line_idx: usize, string: &str) {
        let line_start = self.line_to_char(line_idx);
        self.insert(line_start, &string);
        self.insert_char(line_start + string.len(), '\n')
    }
}