use ropey::RopeSlice;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    pub ln: usize,
    pub col: usize,
}

pub(crate) fn trimmed(rope: RopeSlice) -> RopeSlice {
    let rope_len = rope.len_chars();
    if rope_len == 0 {
        rope
    } else if rope.char(rope_len - 1) == '\n' {
        rope.slice(..rope_len - 1)
    } else {
        rope.slice(..rope_len)
    }
}