
<img width="878" height="232" alt="Screenshot 2026-01-18 at 9 48 44 pm" src="https://github.com/user-attachments/assets/d43e2f73-f081-4d5c-b03f-ad1a9d9e7929" />

### usage

1. run `cargo build --release && mkdir -p ~/.config/tmux && cp target/release/tmux-status ~/.config/tmux/status`.
2. Add this to your tmux config and then reload/restart tmux:

```tmux
set -g status-position "bottom"
set -g status-interval 10
set -g status-style ""

quote_regex="s/'/'\"'\"'/"
window="'#{W:#{$quote_regex:window_name}#,}' '#{active_window_index}'"
pane="'#{$quote_regex:pane_title}' '#{$quote_regex:pane_current_path}'"
session="'#{session_id}' '#{$quote_regex:session_name}' '#{session_color}' '#{client_width}'"
selection_y="'#{selection_start_y}' '#{selection_end_y}'"
selection_x="'#{selection_start_x}' '#{selection_end_x}'"
selection="$selection_y $selection_x"
status_props="$pane $window $session $selection #{window_zoomed_flag}"
status="~/.local/bin/tmux-status $status_props"
set -g status-format[0] "#($status 0 0)"
bind -n MouseDown1StatusDefault run "$status 1 '#{mouse_x}'"
bind -n MouseDrag1StatusDefault run "$status 2 '#{mouse_x}'"
```
