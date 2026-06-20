use std::process::Command;

use chrono::Local;
use tuie::prelude::*;

use crate::block::Block;
use crate::blockrow::{BlockRow, Row};
use crate::options::{Action, Options};
use crate::powerline::Powerline;
use crate::render;
use crate::tabs::Tabs;

const GIT_BRANCH_ICON: &str = "\u{e0a0}";
const UNKNOWN_PATH_ICON: &str = "???";
pub const CLOCK_TAG: &str = "clock";
const POPUP_SESSION: &str = "popup";
pub const ACCENT: Color = Color::BLUE;
const POWERLINE_FILL: Powerline = Powerline::BLOCK;
const POWERLINE_DIVIDER: Powerline = Powerline::BLOCK;
const ENCLOSE_ACTIVE: bool = false;
const MAX_LINES: usize = 2;

fn get_git_branch(pane_path: &str) -> String {
    Command::new("git")
        .current_dir(pane_path)
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()
        .map(|output| {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        })
        .unwrap_or_default()
}

fn open_popup(flag: &'static str) -> impl Fn() + 'static {
    move || {
        if let Ok(exe) = std::env::current_exe() {
            Command::new(exe)
                .args(std::env::args().skip(1))
                .arg(flag)
                .spawn()
                .ok();
        }
    }
}

#[derive(Clone, Copy)]
pub struct Selection {
    pub y_start: i32,
    pub y_end: i32,
    pub x_start: i32,
    pub x_end: i32,
}

impl Selection {
    fn label(&self) -> String {
        if self.y_start != self.y_end {
            format!("{} rows", (self.y_start - self.y_end).abs() + 1)
        } else {
            let cols = (self.x_start - self.x_end).abs() + 1;
            format!("{} col{}", cols, if cols == 1 { "" } else { "s" })
        }
    }
}

pub fn create_right_row(config: &Options, bar_bg: Color) -> Row {
    let selection = config.has_selection.then_some(config.selection);
    let session = match selection {
        Some(sel) => sel.label(),
        None => config.session_title.clone(),
    };
    let fill = Style::new().fg(Color::grey256(0)).bg(ACCENT).bold();
    let block_bg = if config.is_zoomed {
        Color::grey256(8)
    } else {
        Color::grey256(5)
    };
    let time = Local::now().format("%H:%M %d-%b-%y").to_string();

    let mut row = BlockRow::new(bar_bg)
        .right()
        .families(&POWERLINE_FILL, &POWERLINE_DIVIDER)
        .enclose(ENCLOSE_ACTIVE)
        .separator(Style::new().fg(Color::grey256(6)))
        .text(config.pane_title.clone(), Style::new())
        .truncate()
        .flex()
        .push(path_block(&config.pane_path));

    let branch = get_git_branch(&config.pane_path);
    if !branch.is_empty() {
        row = row.text(
            format!("{} {}", GIT_BRANCH_ICON, branch),
            Style::new().bg(block_bg),
        );
    }

    row
        .active(session, fill)
        .push(
            Block::new()
                .span(format!("{} ", time), Style::new().bg(block_bg))
                .tag(CLOCK_TAG)
                .on_click(open_popup("--open-calendar")),
        )
        .row()
}

fn path_block(pane_path: &str) -> Block {
    let filename = if pane_path == "/" {
        Some("/")
    } else {
        std::path::Path::new(pane_path)
            .file_name()
            .and_then(|n| n.to_str())
    };
    match filename {
        Some(name) => Block::new().span(name.to_string(), Style::new()),
        None => Block::new()
            .span(UNKNOWN_PATH_ICON.to_string(), Style::new().fg(Color::RED)),
    }
}

fn empty_row(bar_bg: Color) -> Row {
    BlockRow::new(bar_bg).row()
}

fn current_status_lines() -> usize {
    Command::new("tmux")
        .args(["show", "-gv", "status"])
        .output()
        .ok()
        .map(|output| {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        })
        .map_or(1, |value| match value.as_str() {
            "off" => 0,
            "on" => 1,
            other => other.parse().unwrap_or(1),
        })
}

fn set_status_lines(n: usize) {
    let value = match n {
        0 => "off".to_string(),
        1 => "on".to_string(),
        other => other.to_string(),
    };
    Command::new("tmux")
        .args(["set", "-g", "status", &value])
        .spawn()
        .ok();
}

fn line_offset(counts: &[usize], line: usize) -> Option<(usize, usize)> {
    if line >= counts.len() {
        return None;
    }
    Some((counts[..line].iter().sum(), counts[line]))
}

pub fn render(config: &Options) -> std::io::Result<std::process::ExitCode> {
    if config.session_title == POPUP_SESSION {
        return Ok(std::process::ExitCode::SUCCESS);
    }

    let bar_bg = if config.is_zoomed {
        Color::grey256(5)
    } else {
        Color::grey256(2)
    };
    let width = config.client_size.x;
    let cap = MAX_LINES.clamp(1, 5);
    let tabs = Tabs::new(config.window_idx, &config.windows)
        .active_style(Style::new().fg(Color::grey256(0)).bg(ACCENT).bold())
        .families(&POWERLINE_FILL, &POWERLINE_DIVIDER)
        .enclose(ENCLOSE_ACTIVE)
        .separator(Style::new().fg(Color::grey256(6)))
        .bar_bg(bar_bg);

    let right = create_right_row(config, bar_bg);
    let counts = tabs.partition(width, right.width(), cap);

    if config.line == 0 && cap > 1 && current_status_lines() != counts.len() {
        set_status_lines(counts.len());
    }

    let Some((start, len)) = line_offset(&counts, config.line) else {
        if matches!(config.action, Action::Render) {
            print!(
                "{}",
                render::emit(empty_row(bar_bg), empty_row(bar_bg), width)
            );
        }
        return Ok(std::process::ExitCode::SUCCESS);
    };

    let line_tabs = tabs.line(start, len);
    let line_right = if config.line == counts.len() - 1 {
        right
    } else {
        empty_row(bar_bg)
    };

    match config.action {
        Action::Drag(mouse_x) => line_tabs.drag(bar_bg, width, mouse_x),
        Action::Click(mouse_x) => {
            render::click(line_tabs.row(), line_right, width, mouse_x)
        }
        _ => print!("{}", render::emit(line_tabs.row(), line_right, width)),
    }

    Ok(std::process::ExitCode::SUCCESS)
}
