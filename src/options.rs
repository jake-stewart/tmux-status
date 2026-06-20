use option_parser::{Arg::Flag, OptionParser};
use tuie::prelude::*;

use crate::statusline::Selection;

const NO_SELECTION: i32 = -1;

pub enum Action {
    Render,
    Click(usize),
    Drag(usize),
    OpenCalendar,
    Calendar,
}

pub struct Options {
    pub action: Action,
    pub pane_title: String,
    pub pane_path: String,
    pub windows: Vec<(usize, String)>,
    pub window_idx: usize,
    pub session_title: String,
    pub client_size: Vec2<usize>,
    pub has_selection: bool,
    pub selection: Selection,
    pub is_zoomed: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            action: Action::Render,
            pane_title: String::new(),
            pane_path: String::new(),
            windows: Vec::new(),
            window_idx: 0,
            session_title: String::new(),
            client_size: Vec2::new(0, 0),
            has_selection: false,
            selection: Selection {
                y_start: NO_SELECTION,
                y_end: NO_SELECTION,
                x_start: NO_SELECTION,
                x_end: NO_SELECTION,
            },
            is_zoomed: false,
        }
    }
}

fn parse_coord(value: String) -> i32 {
    value.parse().unwrap_or(NO_SELECTION)
}

impl Options {
    pub fn parse(args: &[String]) -> Result<Self, option_parser::Error> {
        let mut options = Options::default();
        let mut parser = OptionParser::args(args);
        while let Some(arg) = parser.parse_arg()? {
            match arg {
                Flag("open-calendar") => {
                    options.action = Action::OpenCalendar
                }
                Flag("calendar") => options.action = Action::Calendar,
                Flag("click") => {
                    options.action = Action::Click(parser.parse_value()?)
                }
                Flag("drag") => {
                    options.action = Action::Drag(parser.parse_value()?)
                }
                Flag("pane-title") => {
                    options.pane_title = parser.parse_value()?
                }
                Flag("pane-path") => {
                    options.pane_path = parser.parse_value()?
                }
                Flag("window") => {
                    let idx = parser.parse_value()?;
                    let name = parser.parse_value()?;
                    options.windows.push((idx, name));
                }
                Flag("window-index") => {
                    options.window_idx = parser.parse_value()?
                }
                Flag("session") => {
                    options.session_title = parser.parse_value()?
                }
                Flag("width") => {
                    options.client_size.x = parser.parse_value()?
                }
                Flag("height") => {
                    options.client_size.y = parser.parse_value()?
                }
                Flag("selection") => {
                    options.has_selection =
                        parser.parse_value::<String>()? == "1"
                }
                Flag("selection-y") => {
                    options.selection.y_start =
                        parse_coord(parser.parse_value()?);
                    options.selection.y_end =
                        parse_coord(parser.parse_value()?);
                }
                Flag("selection-x") => {
                    options.selection.x_start =
                        parse_coord(parser.parse_value()?);
                    options.selection.x_end =
                        parse_coord(parser.parse_value()?);
                }
                Flag("zoomed") => {
                    options.is_zoomed = parser.parse_value::<String>()? == "1"
                }
                _ => parser.unexpected()?,
            }
        }
        Ok(options)
    }
}
