use std::env;
use std::process::Command;
use unicode_width::UnicodeWidthStr;
use chrono::Local;

// Color constants
const GREY_0: &str = "colour232";
const GREY_2: &str = "colour234";
const GREY_5: &str = "colour237";
const GREY_6: &str = "colour238";

// UI constants
const GIT_BRANCH_ICON: &str = "\u{e0a0}";
const UNKNOWN_PATH_ICON: &str = "???";
const MIN_SPACING: usize = 3;
const NO_SELECTION: i32 = -1;

mod mouse {
    pub const CLICK: i32 = 1;
    pub const DRAG: i32 = 2;
}

mod arg {
    pub const PANE_TITLE: usize = 1;
    pub const PANE_PATH: usize = 2;
    pub const WINDOW_STR: usize = 3;
    pub const WINDOW_IDX: usize = 4;
    pub const SESSION_TITLE: usize = 6;
    pub const SESSION_COLOR: usize = 7;
    pub const CLIENT_WIDTH: usize = 8;
    pub const SELECTION_Y_START: usize = 9;
    pub const SELECTION_Y_END: usize = 10;
    pub const SELECTION_X_START: usize = 11;
    pub const SELECTION_X_END: usize = 12;
    pub const IS_ZOOMED: usize = 13;
    pub const MOUSE_MODE: usize = 14;
    pub const MOUSE_X: usize = 15;
    pub const MIN_REQUIRED: usize = 16;
}

pub fn rgb256(r: u8, g: u8, b: u8) -> u8 {
    16 + r * 36 + g * 6 + b
}

// ============================================================================
// Color State
// ============================================================================

struct ColorState {
    current_fg: String,
    current_bg: String,
    default_fg: String,
    default_bg: String,
}

impl ColorState {
    fn new() -> Self {
        Self {
            current_fg: String::from("default"),
            current_bg: String::from("default"),
            default_fg: String::from("default"),
            default_bg: String::from("default"),
        }
    }

    fn with_background(bg: &str) -> Self {
        Self {
            current_fg: String::from("default"),
            current_bg: String::from("default"),
            default_fg: String::from("default"),
            default_bg: bg.to_string(),
        }
    }
}

// ============================================================================
// Block Span
// ============================================================================

#[derive(Clone)]
struct BlockSpan {
    text: String,
    size: usize,
    attr: Option<&'static str>,
    fg: String,
    bg: String,
}

impl BlockSpan {
    fn new(text: String, state: &ColorState) -> Self {
        let size = text.width();
        Self {
            text,
            size,
            attr: None,
            fg: state.default_fg.clone(),
            bg: state.default_bg.clone(),
        }
    }

    fn bold(mut self) -> Self {
        self.attr = Some("bold");
        self
    }

    fn fg(mut self, fg: &str) -> Self {
        self.fg = fg.to_string();
        self
    }

    fn bg(mut self, bg: &str) -> Self {
        self.bg = bg.to_string();
        self
    }

    fn print(&self, state: &mut ColorState) {
        let format_codes = self.build_format_codes(state);
        print!("{}{}{}", format_codes.0, self.text, format_codes.1);
        
        state.current_fg = self.fg.clone();
        state.current_bg = self.bg.clone();
    }

    fn build_format_codes(&self, state: &ColorState) -> (String, String) {
        let colors_match = state.current_fg
            == self.fg && state.current_bg == self.bg;
        
        if colors_match {
            return self.build_attr_only_codes();
        }
        
        self.build_full_codes(state)
    }

    fn build_attr_only_codes(&self) -> (String, String) {
        if let Some(attr) = self.attr {
            (format!("#[{}]", attr), format!("#[no{}]", attr))
        } else {
            (String::new(), String::new())
        }
    }

    fn build_full_codes(&self, state: &ColorState) -> (String, String) {
        let mut parts = Vec::new();
        
        if state.current_fg != self.fg {
            parts.push(format!("fg={}", self.fg));
        }
        
        if state.current_bg != self.bg {
            parts.push(format!("bg={}", self.bg));
        }
        
        if let Some(attr) = self.attr {
            parts.push(attr.to_string());
        }
        
        let open = format!("#[{}]", parts.join(","));
        let close = self.attr.map(
            |a| format!("#[no{}]", a)).unwrap_or_default();
        
        (open, close)
    }

    fn length(&self) -> usize {
        self.size
    }
}

// ============================================================================
// Block
// ============================================================================

struct Block {
    spans: Vec<BlockSpan>,
    on_click: Option<Box<dyn Fn()>>,
}

impl Block {
    fn new() -> Self {
        Self {
            spans: Vec::new(),
            on_click: None,
        }
    }

    fn add(&mut self, span: BlockSpan) {
        self.spans.push(span);
    }

    fn with_click(mut self, callback: Box<dyn Fn()>) -> Self {
        self.on_click = Some(callback);
        self
    }

    fn length(&self) -> usize {
        self.spans.iter().map(|s| s.length()).sum()
    }

    fn print(&self, state: &mut ColorState) {
        for span in &self.spans {
            span.print(state);
        }
    }

    fn click(&self) {
        if let Some(ref callback) = self.on_click {
            callback();
        }
    }
}

// ============================================================================
// Block Row - Horizontal layout of blocks
// ============================================================================

struct BlockRow {
    blocks: Vec<Block>,
}

impl BlockRow {
    fn new() -> Self {
        Self { blocks: Vec::new() }
    }

    fn add(&mut self, block: Block) {
        self.blocks.push(block);
    }

    fn click(&self, pos: usize) -> bool {
        let mut offset = 0;
        
        for block in &self.blocks {
            let block_end = offset + block.length();
            
            if pos >= offset && pos < block_end {
                block.click();
                return true;
            }
            
            offset = block_end;
        }
        
        false
    }

    fn length(&self) -> usize {
        self.blocks.iter().map(|b| b.length()).sum()
    }

    fn clear(&mut self) {
        self.blocks.clear();
    }

    fn shorten(&mut self, target: usize, from_back: bool) {
        while !self.blocks.is_empty() && self.length() > target {
            if from_back {
                self.blocks.pop();
            } else {
                self.blocks.remove(0);
            }
        }
    }

    fn remove_empty(&mut self) {
        self.blocks.retain(|b| b.length() > 0);
    }

    fn print(&self, state: &mut ColorState) {
        for block in &self.blocks {
            block.print(state);
        }
    }
}

// ============================================================================
// Layout
// ============================================================================

fn resize(
    left_row: &mut BlockRow,
    right_row: &mut BlockRow,
    width: usize
) -> usize {
    let left_len = left_row.length();
    
    if left_len >= width {
        right_row.clear();
        left_row.shorten(width, true);
        return 0;
    }
    
    let mut remaining = width - left_len;
    let right_len = right_row.length();
    
    if remaining < right_len + MIN_SPACING {
        let max_right = width.saturating_sub(left_len + MIN_SPACING);
        right_row.shorten(max_right, false);
        remaining = width - left_len - right_row.length();
    } else {
        remaining -= right_len;
    }
    
    remaining
}

// ============================================================================
// Commands
// ============================================================================

fn exec_getline(cmd: &str) -> String {
    Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .ok()
        .map(|output| String::from_utf8_lossy(&output.stdout)
            .trim().to_string())
        .unwrap_or_default()
}

fn get_git_branch(pane_path: &str) -> String {
    let cmd = format!(
        "(cd '{}' && git rev-parse --abbrev-ref HEAD 2>/dev/null)",
        pane_path
    );
    exec_getline(&cmd)
}

fn spawn_tmux_command(args: &[&str]) {
    Command::new("tmux").args(args).spawn().ok();
}

fn spawn_shell_script(script: &str) {
    Command::new("sh").arg("-c").arg(script).spawn().ok();
}

// ============================================================================
// Color Mapping
// ============================================================================

fn session_color_to_rgb(color_name: &str, session_title: &str) -> String {
    match color_name {
        "red" => format!("colour{}", rgb256(3, 0, 0)),
        "green" => format!("colour{}", rgb256(0, 3, 0)),
        "blue" => format!("colour{}", rgb256(0, 1, 4)),
        "yellow" => format!("colour{}", rgb256(3, 3, 0)),
        "cyan" => format!("colour{}", rgb256(0, 3, 3)),
        "magenta" | "purple" => format!("colour{}", rgb256(3, 0, 3)),
        _ if session_title == "scratch" => String::from("cyan"),
        _ => String::from("brightblack"),
    }
}

// ============================================================================
// Tab Blocks
// ============================================================================

fn create_tab_blocks(
    active_idx: usize,
    window_str: &str,
    color: &str,
    state: &ColorState,
) -> BlockRow {
    let mut row = BlockRow::new();
    let windows: Vec<&str> = window_str
        .split(',').filter(|s| !s.is_empty()).collect();

    for (idx, name) in windows.iter().enumerate() {
        let block = if idx == active_idx {
            create_active_tab(name, color, state)
        } else {
            create_inactive_tab(idx, active_idx, name, state)
        };
        
        row.add(block);
    }

    row.remove_empty();
    row
}

fn create_active_tab(name: &str, color: &str, state: &ColorState) -> Block {
    let mut block = Block::new();
    block.add(
        BlockSpan::new(format!(" {} ", name), state)
            .bold()
            .fg(GREY_0)
            .bg(color)
    );
    block
}

fn create_inactive_tab(
    idx: usize,
    active_idx: usize,
    name: &str,
    state: &ColorState
) -> Block {
    let mut block = Block::new();
    let needs_separator =
        idx > 0 && active_idx != idx && active_idx != idx - 1;

    if needs_separator {
        block.add(BlockSpan::new(String::from("▏"), state).fg(GREY_6));
        block.add(BlockSpan::new(format!("{} ", name), state));
    } else {
        block.add(BlockSpan::new(format!(" {} ", name), state));
    }

    block.with_click(Box::new(move || {
        spawn_tmux_command(&["select-window", "-t", &idx.to_string()]);
    }))
}

// ============================================================================
// Right Status Bar
// ============================================================================

struct RightStatusConfig<'a> {
    pane_title: &'a str,
    pane_path: &'a str,
    session_title: &'a str,
    selection_y_start: i32,
    selection_y_end: i32,
    selection_x_start: i32,
    selection_x_end: i32,
    color: &'a str,
}

fn create_right_row(
    config: RightStatusConfig,
    state: &ColorState
) -> BlockRow {
    let mut row = BlockRow::new();

    row.add(create_pane_title_block(config.pane_title, state));
    row.add(create_path_block(config.pane_path, state));
    
    let branch = get_git_branch(config.pane_path);
    if !branch.is_empty() {
        row.add(create_git_branch_block(&branch, state));
    }

    row.add(create_session_or_selection_block(
        config.session_title,
        config.selection_y_start,
        config.selection_y_end,
        config.selection_x_start,
        config.selection_x_end,
        config.color,
        state,
    ));

    row.add(create_time_block(state));
    row.remove_empty();
    row
}

fn create_pane_title_block(pane_title: &str, state: &ColorState) -> Block {
    let mut block = Block::new();
    block.add(BlockSpan::new(pane_title.to_string(), state));
    block.add(BlockSpan::new(String::from(" ▏"), state).fg(GREY_6));
    block
}

fn create_path_block(pane_path: &str, state: &ColorState) -> Block {
    let mut block = Block::new();
    
    let filename = if pane_path == "/" {
        "/".to_string()
    } else {
        std::path::Path::new(pane_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(UNKNOWN_PATH_ICON)
            .to_string()
    };

    if !filename.is_empty() && filename != UNKNOWN_PATH_ICON {
        block.add(BlockSpan::new(filename, state));
        block.add(BlockSpan::new(" ".to_string(), state));
    } else {
        block.add(BlockSpan::new(
            UNKNOWN_PATH_ICON.to_string(), state).fg("red"));
    }

    block
}

fn create_git_branch_block(branch: &str, state: &ColorState) -> Block {
    let mut block = Block::new();
    block.add(BlockSpan::new(String::from("▏"), state).fg(GREY_6));
    block.add(BlockSpan::new(
        format!("{} {} ", GIT_BRANCH_ICON, branch), state));
    block
}

fn create_session_or_selection_block(
    session_title: &str,
    sel_y_start: i32,
    sel_y_end: i32,
    sel_x_start: i32,
    sel_x_end: i32,
    color: &str,
    state: &ColorState,
) -> Block {
    let mut block = Block::new();

    if sel_y_start != sel_y_end {
        let title = format!("{} rows",
            (sel_y_start.max(sel_y_end) - sel_y_start.min(sel_y_end) + 1));
        block.add(BlockSpan::new(format!(" {} ", title), state)
            .fg(GREY_0).bg(color).bold());
    } else if sel_x_start != NO_SELECTION {
        let num = sel_x_start.max(sel_x_end) - sel_x_start.min(sel_x_end) + 1;
        let title = format!("{} col{}", num, if num == 1 { "" } else { "s" });
        block.add(BlockSpan::new(format!(" {} ", title), state)
            .fg(GREY_0).bg(color).bold());
    } else {
        block.add(BlockSpan::new(format!(" {} ", session_title), state)
            .fg(GREY_0).bg(color).bold());
        block = block.with_click(Box::new(|| {
            spawn_shell_script("~/.config/tmux/popup-switch-session.sh");
        }));
    }

    block
}

fn create_time_block(state: &ColorState) -> Block {
    let mut block = Block::new();
    let time = Local::now().format("%H:%M %d-%b-%y").to_string();
    block.add(BlockSpan::new(format!(" {} ", time), state));
    block.with_click(Box::new(|| {
        spawn_shell_script("~/.config/tmux/popup-cal.sh");
    }))
}

// ============================================================================
// Mouse
// ============================================================================

fn handle_mouse_click(
    left_row: &BlockRow,
    right_row: &BlockRow,
    mouse_x: usize,
    client_width: usize
) {
    if !left_row.click(mouse_x) {
        let right_offset = client_width.saturating_sub(right_row.length());
        right_row.click(mouse_x.saturating_sub(right_offset));
    }
}

fn handle_mouse_drag(left_row: &BlockRow, window_idx: usize, mouse_x: usize) {
    if window_idx >= left_row.blocks.len() {
        return;
    }

    let mut offset = 0;
    
    for (idx, block) in left_row.blocks.iter().enumerate() {
        if idx == window_idx {
            offset += block.length();
            continue;
        }

        let block_end = offset + block.length();
        
        if mouse_x >= offset && mouse_x < block_end {
            perform_window_move(window_idx, idx, mouse_x, offset, left_row);
            return;
        }

        offset = block_end;
    }
}

fn perform_window_move(
    window_idx: usize,
    target_idx: usize,
    mouse_x: usize,
    target_offset: usize,
    left_row: &BlockRow,
) {
    let src_offset: usize = left_row.blocks.iter()
        .take(window_idx).map(|b| b.length()).sum();
    let active_length = left_row.blocks[window_idx].length();
    let target_length = left_row.blocks[target_idx].length();

    if target_offset < src_offset {
        if mouse_x < src_offset + active_length - target_length {
            spawn_tmux_command(
                &["move-window", "-b", "-t", &target_idx.to_string()]);
        }
    } else if mouse_x >= src_offset + target_length {
        spawn_tmux_command(
            &["move-window", "-a", "-t", &target_idx.to_string()]);
    }
}

// ============================================================================
// Arg Parsing
// ============================================================================

struct StatusConfig {
    pane_title: String,
    pane_path: String,
    window_str: String,
    window_idx: usize,
    session_title: String,
    session_color: String,
    client_width: usize,
    selection_y_start: i32,
    selection_y_end: i32,
    selection_x_start: i32,
    selection_x_end: i32,
    is_zoomed: bool,
    mouse_mode: i32,
    mouse_x: usize,
}

impl StatusConfig {
    fn parse_from_args(args: &[String]) -> Option<Self> {
        if args.len() < arg::MIN_REQUIRED {
            return None;
        }

        let (sel_y_start, sel_y_end, sel_x_start, sel_x_end) = 
            if !args[arg::SELECTION_Y_START].is_empty() {(
                args[arg::SELECTION_Y_START].parse().unwrap_or(NO_SELECTION),
                args[arg::SELECTION_Y_END].parse().unwrap_or(NO_SELECTION),
                args[arg::SELECTION_X_START].parse().unwrap_or(NO_SELECTION),
                args[arg::SELECTION_X_END].parse().unwrap_or(NO_SELECTION),
            )} else {(
                NO_SELECTION, NO_SELECTION, NO_SELECTION, NO_SELECTION
            )};

        Some(Self {
            pane_title: args[arg::PANE_TITLE].clone(),
            pane_path: args[arg::PANE_PATH].clone(),
            window_str: args[arg::WINDOW_STR].clone(),
            window_idx: args[arg::WINDOW_IDX].parse().unwrap_or(0),
            session_title: args[arg::SESSION_TITLE].clone(),
            session_color: args[arg::SESSION_COLOR].clone(),
            client_width: args[arg::CLIENT_WIDTH].parse().unwrap_or(0),
            selection_y_start: sel_y_start,
            selection_y_end: sel_y_end,
            selection_x_start: sel_x_start,
            selection_x_end: sel_x_end,
            is_zoomed: args[arg::IS_ZOOMED].parse::<i32>().unwrap_or(0) != 0,
            mouse_mode: args[arg::MOUSE_MODE].parse().unwrap_or(0),
            mouse_x: args[arg::MOUSE_X].parse().unwrap_or(0),
        })
    }
}

// ============================================================================
// Main
// ============================================================================

fn main() {
    let args: Vec<String> = env::args().collect();
    let Some(config) = StatusConfig::parse_from_args(&args) else { return };

    if config.session_title == "popup" {
        return;
    }

    let bg_color = if config.is_zoomed { GREY_5 } else { GREY_2 };
    let mut state = ColorState::with_background(bg_color);
    let color = session_color_to_rgb(
        &config.session_color, &config.session_title);

    // Handle mouse interactions
    if config.mouse_mode == mouse::CLICK || config.mouse_mode == mouse::DRAG {
        let left_row = create_tab_blocks(
            config.window_idx, &config.window_str, &color, &state);
        let right_row = create_right_row(
            RightStatusConfig {
                pane_title: &config.pane_title,
                pane_path: &config.pane_path,
                session_title: &config.session_title,
                selection_y_start: config.selection_y_start,
                selection_y_end: config.selection_y_end,
                selection_x_start: config.selection_x_start,
                selection_x_end: config.selection_x_end,
                color: &color,
            },
            &state,
        );

        if config.mouse_mode == mouse::CLICK {
            handle_mouse_click(
                &left_row, &right_row, config.mouse_x, config.client_width);
        } else {
            handle_mouse_drag(&left_row, config.window_idx, config.mouse_x);
        }
        return;
    }

    // Render status bar
    let mut left_row = create_tab_blocks(
        config.window_idx, &config.window_str, &color, &state);
    left_row.remove_empty();

    let mut right_row = create_right_row(
        RightStatusConfig {
            pane_title: &config.pane_title,
            pane_path: &config.pane_path,
            session_title: &config.session_title,
            selection_y_start: config.selection_y_start,
            selection_y_end: config.selection_y_end,
            selection_x_start: config.selection_x_start,
            selection_x_end: config.selection_x_end,
            color: &color,
        },
        &state,
    );

    let spacing = resize(&mut left_row, &mut right_row, config.client_width);

    print!("#[bg={}]", state.default_bg);
    state.current_bg = state.default_bg.clone();

    left_row.print(&mut state);
    print!("#[bg={}]{:width$}", state.default_bg, "", width = spacing);
    
    state.current_bg = state.default_bg.clone();
    right_row.print(&mut state);
}
