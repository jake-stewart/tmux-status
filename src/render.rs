use std::collections::HashMap;
use std::rc::Rc;

use chord_macro::chord;
use tuie::emulator::Emulator;
use tuie::prelude::*;

use crate::block::Block;
use crate::blockrow::Row;
use crate::tmux::{color_to_tmux, snapshot_to_tmux};

pub const MIN_SPACING: usize = 3;

pub struct Clickable {
    inner: Box<Text>,
    on_click: Option<Rc<dyn Fn()>>,
}

impl Clickable {
    fn new(inner: Box<Text>) -> Box<Self> {
        Box::new(Self {
            inner,
            on_click: None,
        })
    }

    fn on_click(mut self: Box<Self>, callback: Rc<dyn Fn()>) -> Box<Self> {
        self.on_click = Some(callback);
        self
    }
}

impl DelegateWidget for Clickable {
    tuie::delegate_widget!(inner);

    fn override_on_input(&mut self, queue: &mut InputQueue) -> InputResult {
        let Some(event) = queue.peek() else {
            return InputResult::Rejected;
        };
        if let Some(callback) = &self.on_click
            && matches!(event.chord, chord!(LeftClick))
        {
            queue.next();
            callback();
            return InputResult::Handled;
        }
        InputResult::Rejected
    }
}

fn into_widget(
    spans: Vec<(String, Style)>,
    on_click: Option<Rc<dyn Fn()>>,
) -> Box<Clickable> {
    let mut content = StyledString::new();
    for (text, style) in &spans {
        content = content.span(StyledStr {
            text,
            style: *style,
        });
    }
    let widget = Clickable::new(Text::new().content(content));
    match on_click {
        Some(callback) => widget.on_click(callback),
        None => widget,
    }
}

pub fn fit(left: &mut Row, right: &mut Row, width: usize) {
    if left.width() >= width {
        right.clear();
        let trimmed = trim_left(left, width);
        left.set_cut(trimmed);
    } else {
        let left_len = left.width();
        let trimmed = trim_right(right, left_len, width);
        right.set_cut(trimmed);
    }
}

fn trim_left(left: &mut Row, width: usize) -> bool {
    let widths = left.prefix_widths();
    let n = widths.len() - 1;
    let mut keep = n;
    while keep > 0 && widths[keep] > width {
        keep -= 1;
    }
    left.keep_prefix(keep);
    keep < n
}

fn trim_right(right: &mut Row, left_len: usize, width: usize) -> bool {
    if right.first_truncatable() {
        let max_right = width.saturating_sub(left_len);
        let full = right.width();
        if full <= max_right {
            return false;
        }
        right.truncate_first(full - max_right);
        if right.width() <= max_right {
            return false;
        }
        right.drop_front(1);
        trim_front(right, left_len, width);
        return true;
    }
    trim_front(right, left_len, width)
}

fn trim_front(right: &mut Row, left_len: usize, width: usize) -> bool {
    let max_right = width.saturating_sub(left_len + MIN_SPACING);
    let widths = right.suffix_widths();
    let n = widths.len() - 1;
    let mut drop = 0;
    while drop < n && widths[drop] > max_right {
        drop += 1;
    }
    right.drop_front(drop);
    drop > 0
}

pub struct Layout {
    pub tabs: Vec<WidgetId<Clickable>>,
    tagged: HashMap<&'static str, WidgetId<Clickable>>,
}

impl Layout {
    pub fn tagged(&self, tag: &str) -> Option<WidgetId<Clickable>> {
        self.tagged.get(tag).copied()
    }
}

pub fn build_root(
    left: Vec<Block>,
    right: Vec<Block>,
    bar_bg: Color,
) -> (Box<Pane>, Layout) {
    let mut root = Pane::new().horizontal().style(Style::new().bg(bar_bg));
    let mut layout = Layout {
        tabs: Vec::new(),
        tagged: HashMap::new(),
    };
    let flexed = left.iter().chain(&right).any(Block::flexed);

    for block in left {
        root = add_block(root, &mut layout, block, true);
    }
    if !flexed {
        root = root.child(Pane::new().flex(1));
    }
    for block in right {
        root = add_block(root, &mut layout, block, false);
    }

    (root, layout)
}

fn add_block(
    root: Box<Pane>,
    layout: &mut Layout,
    block: Block,
    tab: bool,
) -> Box<Pane> {
    let parts = block.into_parts();
    let leading_bg = parts.spans.first().and_then(|(_, style)| style.get_bg());
    let tag = parts.tag;
    let flex = parts.flex.map(|place| (place, leading_bg));
    let widget = into_widget(parts.spans, parts.on_click);
    let id = widget.get_id();
    if tab {
        layout.tabs.push(id);
    }
    if let Some(tag) = tag {
        layout.tagged.insert(tag, id);
    }
    root.child(flex_node(widget, flex))
}

fn flex_node(
    widget: Box<Clickable>,
    flex: Option<(Place, Option<Color>)>,
) -> Box<dyn Widget> {
    match flex {
        Some((place, bg)) => {
            let mut pane = Pane::new().horizontal().flex(1).x_place(place);
            if let Some(bg) = bg {
                pane = pane.style(Style::new().bg(bg));
            }
            pane.child(widget)
        }
        None => widget,
    }
}

pub fn emit(mut left: Row, mut right: Row, width: usize) -> String {
    let bar_bg = left.bar_bg();
    fit(&mut left, &mut right, width);
    let (mut root, _) = build_root(left.render(), right.render(), bar_bg);
    let emu = Emulator::new(&mut *root, Vec2::new(width as u16, 1));
    let snapshot = emu.get_snapshot();
    let bg = color_to_tmux(Some(bar_bg), "default");
    snapshot_to_tmux(&snapshot, &bg)
}

pub fn click(mut left: Row, mut right: Row, width: usize, mouse_x: usize) {
    let bar_bg = left.bar_bg();
    fit(&mut left, &mut right, width);
    let (mut root, _) = build_root(left.render(), right.render(), bar_bg);
    let mut emu = Emulator::new(&mut *root, Vec2::new(width as u16, 1));
    let down =
        Chord::new(Trigger::MouseDown(MouseButton::Left), Modifiers::new());
    let up = Chord::new(Trigger::MouseUp(MouseButton::Left), Modifiers::new());
    let pos = Vec2::new(mouse_x as i32, 0);
    emu.update(
        &mut *root,
        &[
            RuntimeEvent::input_at(down, pos),
            RuntimeEvent::input_at(up, pos),
        ],
    );
}
