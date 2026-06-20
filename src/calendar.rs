use chrono::{Datelike, Local, NaiveDate, Weekday};
use tuie::prelude::*;

use crate::options::Options;
use crate::popup;
use crate::statusline::{ACCENT, CLOCK_TAG, create_right_row};
use crate::tabs::Tabs;

const WEEK_START: Weekday = Weekday::Sun;
const LABELS: [&str; 7] = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];
const GRID_WIDTH: usize = 20;
const CAL_WIDTH: i32 = 28;

fn column(weekday: Weekday) -> usize {
    (weekday.num_days_from_sunday() + 7 - WEEK_START.num_days_from_sunday()) as usize % 7
}

fn weekdays() -> String {
    let start = WEEK_START.num_days_from_sunday() as usize;
    (0..7)
        .map(|i| LABELS[(start + i) % 7])
        .collect::<Vec<_>>()
        .join(" ")
}

fn days_in_month(year: i32, month: u32) -> u32 {
    let (next_year, next_month) =
        if month == 12 { (year + 1, 1) } else { (year, month + 1) };
    NaiveDate::from_ymd_opt(next_year, next_month, 1)
        .unwrap()
        .pred_opt()
        .unwrap()
        .day()
}

fn weeks(year: i32, month: u32) -> Vec<[Option<u32>; 7]> {
    let first = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let mut col = column(first.weekday());
    let mut weeks = Vec::new();
    let mut week = [None; 7];
    for day in 1..=days_in_month(year, month) {
        week[col] = Some(day);
        col += 1;
        if col == 7 {
            weeks.push(week);
            week = [None; 7];
            col = 0;
        }
    }
    if col != 0 {
        weeks.push(week);
    }
    weeks
}

fn centered(text: &str) -> String {
    let pad = GRID_WIDTH.saturating_sub(text.chars().count()) / 2;
    format!("{}{}", " ".repeat(pad), text)
}

fn week_row(week: &[Option<u32>; 7], today: Option<u32>, color: Color) -> StyledString {
    let mut row = StyledString::new();
    for (col, day) in week.iter().enumerate() {
        let cell = match day {
            Some(n) => format!("{:>2}", n),
            None => "  ".to_string(),
        };
        if *day == today {
            row = row
                .span(StyledStr::new("▐").fg(color))
                .span(StyledStr::new(&cell).fg(Color::grey256(0)).bg(color).bold())
                .span(StyledStr::new("▌").fg(color));
        } else {
            if col > 0 && week[col - 1] != today {
                row = row.span(" ");
            }
            row = row.span(cell.as_str());
        }
    }
    row
}

struct Calendar {
    inner: Box<Pane>,
}

impl Calendar {
    fn new() -> Box<Self> {
        let now = Local::now();
        let today = Some(now.day());
        let weeks = weeks(now.year(), now.month());

        let mut inner = Pane::new().vertical().horizontal_padding(3);
        inner.add_child(
            Text::new().content(centered(&now.format("%B %Y").to_string())),
        );
        inner.add_child(Text::new().content(weekdays()));
        for week in &weeks {
            inner.add_child(Text::new().content(week_row(week, today, ACCENT)));
        }
        Box::new(Self { inner })
    }
}

impl DelegateWidget for Calendar {
    tuie::delegate_widget!(inner);

    fn override_on_input(&mut self, queue: &mut InputQueue) -> InputResult {
        if queue.peek().is_some() {
            queue.next();
            tuie::quit(0);
            InputResult::Handled
        } else {
            InputResult::Rejected
        }
    }
}

pub fn spawn_popup(
    config: &Options,
) -> std::io::Result<std::process::ExitCode> {
    let bar_bg = if config.is_zoomed {
        Color::grey256(5)
    } else {
        Color::grey256(2)
    };
    let left = Tabs::new(config.window_idx, &config.windows)
        .bar_bg(bar_bg)
        .row();
    let right = create_right_row(config, bar_bg);
    let exe = std::env::current_exe()?;
    let now = Local::now();
    let rows = weeks(now.year(), now.month()).len() as i32 + 2;
    let height = rows + 3;
    let status = popup::Popup {
        command: format!(
            "'{}' --calendar --session '{}'",
            exe.display(),
            config.session_title
        ),
        size: Vec2::new(CAL_WIDTH, height),
        client_size: config.client_size.map(|n| n as i32),
        anchor: Some(popup::Anchor {
            left,
            right,
            tag: CLOCK_TAG,
        }),
    }
    .open()?;
    Ok(if status.success() {
        std::process::ExitCode::SUCCESS
    } else {
        std::process::ExitCode::FAILURE
    })
}

pub fn run() -> std::io::Result<std::process::ExitCode> {
    tuie::start_tui(Calendar::new())
}
