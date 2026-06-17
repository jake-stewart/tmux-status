use std::process::Command;

use tuie::emulator::Emulator;
use tuie::prelude::*;

use crate::block::Block;
use crate::blockrow::{BlockRow, Row};
use crate::powerline::Powerline;
use crate::render::{Clickable, build_root};

fn spawn_tmux_command(args: &[&str]) {
    Command::new("tmux").args(args).spawn().ok();
}

pub struct Tabs<'a> {
    active: usize,
    windows: &'a [String],
    active_style: Style,
    separator: Style,
    bar_bg: Color,
    fill: &'static Powerline,
    divider: &'static Powerline,
    enclose: bool,
}

impl<'a> Tabs<'a> {
    pub fn new(active: usize, windows: &'a [String]) -> Self {
        Self {
            active,
            windows,
            active_style: Style::new(),
            separator: Style::new(),
            bar_bg: Color::Background,
            fill: &Powerline::BLOCK,
            divider: &Powerline::BLOCK,
            enclose: false,
        }
    }

    pub fn active_style(mut self, style: Style) -> Self {
        self.active_style = style;
        self
    }

    pub fn families(
        mut self,
        fill: &'static Powerline,
        divider: &'static Powerline,
    ) -> Self {
        self.fill = fill;
        self.divider = divider;
        self
    }

    pub fn enclose(mut self, enclose: bool) -> Self {
        self.enclose = enclose;
        self
    }

    pub fn separator(mut self, style: Style) -> Self {
        self.separator = style;
        self
    }

    pub fn bar_bg(mut self, color: Color) -> Self {
        self.bar_bg = color;
        self
    }

    pub fn row(&self) -> Row {
        let mut row = BlockRow::new(self.bar_bg)
            .families(self.fill, self.divider)
            .enclose(self.enclose)
            .separator(self.separator);
        for (idx, name) in self.windows.iter().enumerate() {
            row = if idx == self.active {
                row.active(name.clone(), self.active_style)
            } else {
                row.push(Block::new().span(name.clone(), Style::new()).on_click(
                    move || {
                        spawn_tmux_command(&[
                            "select-window",
                            "-t",
                            &idx.to_string(),
                        ]);
                    },
                ))
            };
        }
        row.row()
    }

    pub fn blocks(&self) -> Vec<Block> {
        self.row().render()
    }

    pub fn drag(&self, bar_bg: Color, width: usize, mouse_x: usize) {
        let (mut root, layout) = build_root(self.blocks(), Vec::new(), bar_bg);
        Emulator::new(&mut *root, Vec2::new(width as u16, 1));
        self.handle_drag(&root, &layout.tabs, mouse_x);
    }

    fn handle_drag(
        &self,
        root: &Pane,
        tabs: &[WidgetId<Clickable>],
        mouse_x: usize,
    ) {
        let rect = |i: usize| {
            tabs.get(i)
                .and_then(|&id| root.get_widget(id))
                .map(|w| w.get_rect())
        };

        let Some(src) = rect(self.active) else {
            return;
        };
        let src_x = src.pos.x.max(0) as usize;
        let active_length = src.size.x as usize;

        for idx in 0..tabs.len() {
            if idx == self.active {
                continue;
            }

            let Some(rect) = rect(idx) else {
                continue;
            };
            let target_x = rect.pos.x.max(0) as usize;
            let target_length = rect.size.x as usize;

            if mouse_x < target_x || mouse_x >= target_x + target_length {
                continue;
            }

            if target_x < src_x {
                if mouse_x < src_x + active_length - target_length {
                    spawn_tmux_command(&[
                        "move-window",
                        "-b",
                        "-t",
                        &idx.to_string(),
                    ]);
                }
            } else if mouse_x >= src_x + target_length {
                spawn_tmux_command(&["move-window", "-a", "-t", &idx.to_string()]);
            }
            return;
        }
    }
}
