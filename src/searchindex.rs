use crate::searchdirection;

pub struct SearchIndex {
    pub x_index     : usize,
    pub y_index     : usize,
    pub x_direction : Option<searchdirection::SearchDirection>,
    pub y_direction : Option<searchdirection::SearchDirection>,
}

impl SearchIndex {
    pub fn new() -> Self {
        Self {
            x_index     : 0,
            y_index     : 0,
            x_direction : None,
            y_direction : None,
        }
    }

    pub fn reset(&mut self) {
        self.y_index     = 0;
        self.x_index     = 0;
        self.y_direction = None;
        self.x_direction = None;
    }
}