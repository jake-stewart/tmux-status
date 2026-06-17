mod app;
mod block;
mod blockrow;
mod calendar;
mod options;
mod popup;
mod powerline;
mod render;
mod statusline;
mod tabs;
mod tmux;

#[cfg(test)]
mod tests;

pub fn crash_log(message: &str) {
    let dir = std::env::var("XDG_STATE_HOME").unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_default();
        format!("{home}/.local/state")
    });
    let dir = format!("{dir}/tmux-status");
    let _ = std::fs::create_dir_all(&dir);
    let _ = std::fs::write(format!("{dir}/crash.log"), format!("{message}\n"));
}

fn main() -> std::io::Result<std::process::ExitCode> {
    std::panic::set_hook(Box::new(|info| {
        crash_log(&info.to_string());
    }));

    let args = option_parser::read_os_args();
    app::run(&args[1..])
}
