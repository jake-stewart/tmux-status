# tmux-status

https://github.com/user-attachments/assets/62c633a6-4831-4872-83d1-c4dc3cd6786c

my tmux statusline, rendered with [tuie](https://github.com/jake-stewart/tuie).

tmux calls this program on every redraw, passing the current pane and session
state through flags, and displays the output markup.

this repo should be easy to fork and make your own, with `statusline.rs` being
a good place to start.

### build

```sh
cargo build --release
cp target/release/tmux-status ~/.config/tmux/status
```

the `tmux.conf` below assumes the binary lives at `~/.config/tmux/status`.

### tmux.conf

```tmux
Q="s/'/'\"'\"'/"  # escape quotes regex

# build the list of data our statusline needs. we must avoid
# querying from the program directly as that can lead to stale data
status="~/.config/tmux/status"
status="$status #{W:--window '#{$Q:window_name}' }"
status="$status --window-index '#{$Q:active_window_index}'"
status="$status --pane-title '#{$Q:pane_title}'"
status="$status --pane-path '#{$Q:pane_current_path}'"
status="$status --session '#{$Q:session_name}'"
status="$status --width '#{$Q:client_width}'"
status="$status --height '#{$Q:client_height}'"
status="$status --selection '#{$Q:selection_present}'"
status="$status --selection-x '#{$Q:selection_start_x}' '#{$Q:selection_end_x}'"
status="$status --selection-y '#{$Q:selection_start_y}' '#{$Q:selection_end_y}'"
status="$status --zoomed '#{$Q:window_zoomed_flag}'"

# set our tmux statusline format to the output of our program
set -g status-format[0] "#($status)"

# it can handle clicks and drags
bind -n MouseDown1StatusDefault run "$status --click '#{mouse_x}'"
bind -n MouseDrag1StatusDefault run "$status --drag '#{mouse_x}'"
```

clicking the clock opens a calendar popup. blocks can spawn popups or trigger
any other custom behaviour from their click handlers.
