use tuie::prelude::*;

pub fn color_to_tmux(color: Option<Color>, default: &str) -> String {
    match color {
        None | Some(Color::Foreground) | Some(Color::Background) => {
            default.to_string()
        }
        Some(Color::Indexed(n)) => format!("colour{}", n),
        Some(Color::Rgb(r, g, b)) => format!("#{:02x}{:02x}{:02x}", r, g, b),
    }
}

fn style_to_tmux(style: &Style, bar_bg: &str) -> String {
    let fg = color_to_tmux(style.get_fg(), "default");
    let bg = color_to_tmux(style.get_bg(), bar_bg);
    let mut s = format!("#[fg={},bg={}", fg, bg);
    if style.has_bold() {
        s.push_str(",bold");
    } else {
        s.push_str(",nobold");
    }
    s.push(']');
    s
}

pub fn snapshot_to_tmux(snapshot: &StyledString, bar_bg: &str) -> String {
    let mut out = String::new();
    for (chunk, style) in snapshot.iter_chunks(..) {
        out.push_str(&style_to_tmux(&style, bar_bg));
        out.push_str(chunk);
    }
    out
}
