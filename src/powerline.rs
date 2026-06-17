use sign::Sign;

pub struct Powerline {
    pub divider_left: &'static str,
    pub divider_right: &'static str,
    pub filled_left: &'static str,
    pub filled_right: &'static str,
}

#[allow(dead_code)]
impl Powerline {
    pub const TRIANGLE: Powerline = Powerline {
        divider_left: "\u{E0B3}",
        divider_right: "\u{E0B1}",
        filled_left: "\u{E0B2}",
        filled_right: "\u{E0B0}",
    };

    pub const CIRCLE: Powerline = Powerline {
        divider_left: "\u{E0B7}",
        divider_right: "\u{E0B5}",
        filled_left: "\u{E0B6}",
        filled_right: "\u{E0B4}",
    };

    pub const LOWER_TRIANGLE: Powerline = Powerline {
        divider_left: "\u{E0BB}",
        divider_right: "\u{E0B9}",
        filled_left: "\u{E0BA}",
        filled_right: "\u{E0B8}",
    };

    pub const UPPER_TRIANGLE: Powerline = Powerline {
        divider_left: "\u{E0BF}",
        divider_right: "\u{E0BD}",
        filled_left: "\u{E0BE}",
        filled_right: "\u{E0BC}",
    };

    pub const FLAME: Powerline = Powerline {
        divider_left: "\u{E0C3} ",
        divider_right: "\u{E0C1} ",
        filled_left: "\u{E0C2} ",
        filled_right: "\u{E0C0} ",
    };

    pub const PIXEL_BIG: Powerline = Powerline {
        divider_left: "\u{E0C7} ",
        divider_right: "\u{E0C6} ",
        filled_left: "\u{E0C7} ",
        filled_right: "\u{E0C6} ",
    };

    pub const PIXEL_SMALL: Powerline = Powerline {
        divider_left: "\u{E0C5} ",
        divider_right: "\u{E0C4} ",
        filled_left: "\u{E0C5} ",
        filled_right: "\u{E0C4} ",
    };

    pub const BLOCK: Powerline = Powerline {
        divider_left: "\u{258F}",
        divider_right: "\u{258F}",
        filled_left: " ",
        filled_right: " ",
    };

    pub const fn divider(&self, dir: Sign) -> &'static str {
        match dir {
            Sign::Positive => self.divider_right,
            Sign::Negative => self.divider_left,
        }
    }

    pub const fn filled(&self, dir: Sign) -> &'static str {
        match dir {
            Sign::Positive => self.filled_right,
            Sign::Negative => self.filled_left,
        }
    }
}
