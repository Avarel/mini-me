#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cursor {
    pub ln: usize,
    pub col: usize,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
    pub focus: Cursor,
    pub anchor: Option<Cursor>
}

impl Selection {
    /// Remove the anchor if it is on the focus.
    pub fn fix_anchor(&mut self) {
        if self.anchor == Some(self.focus) {
            self.anchor = None;
        }
    }

    /// Anchor if there was not already an anchor, or unanchor.
    pub fn set_anchor(&mut self, anchored: bool) {
        if anchored {
            if self.anchor == None {
                self.anchor = Some(self.focus);
            }
        } else {
            self.anchor = None
        }
    }
}