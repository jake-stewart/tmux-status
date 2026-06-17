use std::process::{Command, ExitStatus};

use tuie::emulator::Emulator;
use tuie::prelude::*;

use crate::blockrow::Row;
use crate::render::{build_root, fit};

pub struct Popup {
    pub command: String,
    pub size: Vec2<i32>,
    pub client_size: Vec2<i32>,
    pub anchor: Option<Anchor>,
}

pub struct Anchor {
    pub left: Row,
    pub right: Row,
    pub tag: &'static str,
}

impl Popup {
    pub fn open(self) -> std::io::Result<ExitStatus> {
        let client_w = self.client_size.x;
        let (block_x, block_len) = self
            .anchor
            .and_then(|anchor| anchor.locate(client_w as usize))
            .unwrap_or((client_w - self.size.x, self.size.x));
        let x = (block_x + (block_len - self.size.x) / 2)
            .clamp(0, (client_w - self.size.x).max(0));
        Command::new("tmux")
            .args(["display-popup", "-E"])
            .args(["-w", &self.size.x.to_string()])
            .args(["-h", &self.size.y.to_string()])
            .args(["-x", &x.to_string()])
            .args(["-y", &(self.client_size.y - 1).to_string()])
            .arg(&self.command)
            .status()
    }
}

impl Anchor {
    fn locate(self, client_width: usize) -> Option<(i32, i32)> {
        let Anchor { mut left, mut right, tag } = self;
        fit(&mut left, &mut right, client_width);
        let (mut root, layout) =
            build_root(left.render(), right.render(), Color::Background);
        let id = layout.tagged(tag)?;
        Emulator::new(&mut *root, Vec2::new(client_width as u16, 1));
        let rect = root.get_widget(id)?.get_rect();
        Some((rect.pos.x.max(0), rect.size.x as i32))
    }
}
