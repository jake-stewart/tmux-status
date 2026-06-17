use crate::calendar;
use crate::crash_log;
use crate::options::{Action, Options};
use crate::statusline;

pub fn run(args: &[String]) -> std::io::Result<std::process::ExitCode> {
    let config = match Options::parse(args) {
        Ok(config) => config,
        Err(error) => {
            crash_log(&error.to_string());
            eprintln!("{error}");
            return Ok(std::process::ExitCode::FAILURE);
        }
    };

    match config.action {
        Action::OpenCalendar => calendar::spawn_popup(&config),
        Action::Calendar => calendar::run(),
        _ => statusline::render(&config),
    }
}
