use sign::Sign;
use tuie::prelude::*;

use crate::block::{Block, block_len};
use crate::powerline::Powerline;

#[derive(Clone, Copy)]
struct Decor {
    side: Sign,
    bar_bg: Color,
    separator: Style,
    fill: &'static Powerline,
    divider: &'static Powerline,
    enclose: bool,
}

pub struct BlockRow {
    decor: Decor,
    blocks: Vec<Block>,
}

pub struct Row {
    decor: Decor,
    blocks: Vec<Block>,
    cut: bool,
}

type Span = (String, Style);

struct Edge {
    trail: Option<Span>,
    lead: Option<Span>,
    pad: Option<Span>,
}

fn space() -> Span {
    (String::from(" "), Style::new())
}

fn span_width(span: &Option<Span>) -> usize {
    span.as_ref().map_or(0, |(text, _)| tuie::display_width(text))
}

fn inherit_bg(style: Style, fallback: Option<Color>) -> Style {
    match (style.get_bg(), fallback) {
        (None, Some(bg)) => style.bg(bg),
        _ => style,
    }
}

impl BlockRow {
    pub fn new(bar_bg: Color) -> Self {
        Self {
            decor: Decor {
                side: Sign::Positive,
                bar_bg,
                separator: Style::new(),
                fill: &Powerline::BLOCK,
                divider: &Powerline::BLOCK,
                enclose: false,
            },
            blocks: Vec::new(),
        }
    }

    pub fn right(mut self) -> Self {
        self.decor.side = Sign::Negative;
        self
    }

    #[cfg(test)]
    pub fn powerline(mut self, powerline: &'static Powerline) -> Self {
        self.decor.fill = powerline;
        self.decor.divider = powerline;
        self
    }

    pub fn families(
        mut self,
        fill: &'static Powerline,
        divider: &'static Powerline,
    ) -> Self {
        self.decor.fill = fill;
        self.decor.divider = divider;
        self
    }

    pub fn enclose(mut self, enclose: bool) -> Self {
        self.decor.enclose = enclose;
        self
    }

    pub fn separator(mut self, style: Style) -> Self {
        self.decor.separator = style;
        self
    }

    pub fn text(mut self, text: impl Into<String>, style: Style) -> Self {
        self.blocks.push(Block::new().span(text, style));
        self
    }

    pub fn top(mut self, text: impl Into<String>, fill: Style) -> Self {
        let block = self.decor.capped(text.into(), fill).fill(fill);
        self.blocks.push(block);
        self
    }

    pub fn active(self, text: impl Into<String>, fill: Style) -> Self {
        if self.decor.enclose {
            self.top(text, fill)
        } else {
            self.text(text, fill)
        }
    }

    pub fn push(mut self, block: Block) -> Self {
        self.blocks.push(block);
        self
    }

    pub fn flex(mut self) -> Self {
        let place = match self.decor.side {
            Sign::Positive => Place::Start,
            Sign::Negative => Place::End,
        };
        if let Some(block) = self.blocks.pop() {
            self.blocks.push(block.flex(place));
        }
        self
    }

    pub fn truncate(mut self) -> Self {
        if let Some(block) = self.blocks.pop() {
            self.blocks.push(block.truncate());
        }
        self
    }

    pub fn row(self) -> Row {
        Row {
            decor: self.decor,
            blocks: self.blocks,
            cut: false,
        }
    }

    #[cfg(test)]
    pub fn blocks(self) -> Vec<Block> {
        decorate(&self.decor, self.blocks, false)
    }
}

impl Row {
    pub fn render(self) -> Vec<Block> {
        decorate(&self.decor, self.blocks, self.cut)
    }

    pub fn bar_bg(&self) -> Color {
        self.decor.bar_bg
    }

    pub(crate) fn width(&self) -> usize {
        block_len(&decorate(&self.decor, self.blocks.clone(), self.cut))
    }

    pub(crate) fn prefix_widths(&self) -> Vec<usize> {
        let n = self.blocks.len();
        let mut widths = vec![0; n + 1];
        let mut base = 0;
        for i in 0..n {
            base += self.contrib(i, i == 0);
            widths[i + 1] = base + self.seal_delta(i);
        }
        widths
    }

    pub(crate) fn suffix_widths(&self) -> Vec<usize> {
        let n = self.blocks.len();
        let mut widths = vec![0; n + 1];
        let mut tail = 0;
        for d in (0..n).rev() {
            widths[d] = self.contrib(d, true) + tail;
            if d > 0 {
                tail += self.contrib(d, false);
            }
        }
        widths
    }

    fn contrib(&self, i: usize, first: bool) -> usize {
        let block = &self.blocks[i];
        let prev = (!first).then(|| &self.blocks[i - 1]);
        let edge = self.decor.edge(usize::from(!first), prev, block, &[]);
        let mut w = block.width()
            + span_width(&edge.lead)
            + span_width(&edge.pad)
            + span_width(&edge.trail);
        if first
            && self.decor.on_left()
            && block.filled()
            && !self.decor.flat(self.decor.fill)
        {
            w = w.saturating_sub(block.leading_width());
        }
        w
    }

    fn seal_delta(&self, i: usize) -> usize {
        let block = &self.blocks[i];
        if !self.decor.on_left() || block.filled() {
            return 0;
        }
        let seam = block
            .trailing_bg()
            .is_some_and(|bg| bg != self.decor.bar_bg);
        if !seam {
            return 0;
        }
        let glyph = self.decor.fill.filled(self.decor.side);
        1 + if glyph == " " { 0 } else { tuie::display_width(glyph) }
    }

    pub(crate) fn clear(&mut self) {
        self.blocks.clear();
        self.cut = false;
    }

    pub(crate) fn set_cut(&mut self, cut: bool) {
        self.cut = cut;
    }

    pub(crate) fn keep_prefix(&mut self, keep: usize) {
        self.blocks.truncate(keep);
    }

    pub(crate) fn drop_front(&mut self, count: usize) {
        self.blocks.drain(..count);
    }

    pub(crate) fn first_truncatable(&self) -> bool {
        self.blocks.first().is_some_and(Block::truncatable)
    }

    #[cfg(test)]
    pub(crate) fn last_width(&self) -> usize {
        decorate(&self.decor, self.blocks.clone(), false)
            .last()
            .map_or(0, Block::width)
    }

    pub(crate) fn truncate_first(&mut self, over: usize) {
        if let Some(first) = self.blocks.first_mut() {
            let target = first.width().saturating_sub(over);
            first.truncate_to(target);
        }
    }
}

fn decorate(decor: &Decor, items: Vec<Block>, cut: bool) -> Vec<Block> {
    let fill_flat = decor.flat(decor.fill);
    let topped: Vec<usize> = items
        .iter()
        .enumerate()
        .filter(|(_, b)| b.filled())
        .map(|(i, _)| i)
        .collect();
    let mut blocks = Vec::new();
    let mut pending: Option<Block> = None;
    for (idx, mut content) in items.into_iter().enumerate() {
        let pill = content.filled();
        let edge = decor.edge(idx, pending.as_ref(), &content, &topped);
        let cut_lead = cut && idx == 0 && !decor.on_left();
        if !cut_lead {
            if let Some((text, style)) = edge.pad {
                content = content.prepend(text, style);
            }
            if let Some((text, style)) = edge.lead {
                let style = inherit_bg(style, content.leading_bg());
                content = content.prepend(text, style);
            }
        }
        if idx == 0 && pill && !fill_flat && decor.on_left() {
            content.uncap();
        }
        if let Some(mut previous) = pending.take() {
            decor.weld(&mut previous, &mut content);
            blocks.push(match edge.trail {
                Some((text, style)) => {
                    let style = inherit_bg(style, previous.trailing_bg());
                    previous.span(text, style)
                }
                None => previous,
            });
        }
        pending = Some(content);
    }
    if let Some(previous) = pending {
        blocks.push(previous);
    }
    blocks.retain(|b| !b.is_empty());
    if decor.on_left() {
        decor.seal_trailing(&mut blocks);
    }
    if cut {
        decor.heal_cut(&mut blocks);
    }
    blocks
}

impl Decor {
    fn capped(&self, text: String, fill: Style) -> Block {
        let inner = format!(" {} ", text);
        if self.flat(self.fill) {
            return Block::new().span(inner, fill);
        }
        let glyph = self.fill.filled(self.side);
        let seg = fill.get_bg().unwrap_or(self.bar_bg);
        if self.enclose {
            let cap = Style::new().fg(seg).bg(self.bar_bg);
            return Block::new()
                .span(self.fill.filled_left, cap)
                .span(inner, fill)
                .span(self.fill.filled_right, cap);
        }
        let lead = Style::new().fg(self.bar_bg).bg(seg);
        let trail = Style::new().fg(seg).bg(self.bar_bg);
        let (left, right) = match self.side {
            Sign::Positive => (lead, trail),
            Sign::Negative => (trail, lead),
        };
        Block::new()
            .span(glyph, left)
            .span(inner, fill)
            .span(glyph, right)
    }

    fn on_left(&self) -> bool {
        matches!(self.side, Sign::Positive)
    }

    fn weld(&self, previous: &mut Block, content: &mut Block) {
        match (previous.filled(), content.filled()) {
            (false, true) => {
                if let Some(bg) = previous.trailing_bg() {
                    content.recolor_leading_seam(self.bar_bg, bg);
                }
            }
            (true, false) => {
                if let Some(bg) = content.leading_bg() {
                    previous.recolor_trailing_seam(self.bar_bg, bg);
                }
            }
            _ => {}
        }
    }

    fn seal_trailing(&self, blocks: &mut Vec<Block>) {
        let Some(last) = blocks.last_mut() else {
            return;
        };
        if last.filled() {
            return;
        }
        let Some(bg) = last.trailing_bg().filter(|&bg| bg != self.bar_bg) else {
            return;
        };
        let glyph = self.fill.filled(self.side);
        if last.flexed() {
            last.pad_trailing();
            if glyph != " " {
                blocks.push(
                    Block::new().span(glyph, Style::new().fg(bg).bg(self.bar_bg)),
                );
            }
        } else {
            last.cap_trailing(glyph, self.bar_bg);
        }
    }

    fn heal_cut(&self, blocks: &mut [Block]) {
        if self.on_left() {
            if let Some(last) = blocks.last_mut() {
                last.recolor_trailing_bg(self.bar_bg);
            }
        } else if let Some(first) = blocks.first_mut() {
            match first.leading_bg() {
                None => {}
                Some(lbg)
                    if !first.filled() && Some(lbg) == first.trailing_bg() =>
                {
                    first.cap_leading(self.fill.filled(self.side), self.bar_bg);
                }
                Some(_) => first.recolor_leading_bg(self.bar_bg),
            }
        }
    }

    fn edge(
        &self,
        idx: usize,
        pending: Option<&Block>,
        content: &Block,
        topped: &[usize],
    ) -> Edge {
        let first = idx == 0;
        let prev_pill = pending.is_some_and(Block::filled);
        if content.filled() {
            return Edge {
                trail: (!first).then(space),
                lead: None,
                pad: None,
            };
        }
        if first || prev_pill {
            return Edge {
                trail: None,
                lead: Some(space()),
                pad: None,
            };
        }
        let dir = if self.enclose {
            self.nearest_dir(topped, idx - 1)
        } else {
            self.side
        };
        let lbg = pending.and_then(Block::trailing_bg).unwrap_or(self.bar_bg);
        let rbg = content.leading_bg().unwrap_or(self.bar_bg);
        let trail = Some((String::from(" "), self.separator));
        if lbg != rbg {
            let glyph = self.fill.filled(dir);
            let (fg, bg) = match self.side {
                Sign::Positive => (lbg, rbg),
                Sign::Negative => (rbg, lbg),
            };
            return Edge {
                trail,
                lead: (glyph != " ")
                    .then(|| (glyph.to_string(), Style::new().fg(fg).bg(bg))),
                pad: Some((String::from(" "), Style::new().bg(rbg))),
            };
        }
        Edge {
            trail,
            lead: Some((self.divider_lead(dir), self.separator)),
            pad: None,
        }
    }

    fn nearest_dir(&self, topped: &[usize], gap: usize) -> Sign {
        let dist = |t: &usize| (2 * *t as isize - 2 * gap as isize - 1).abs();
        match topped.iter().copied().min_by_key(dist) {
            Some(t) if t > gap => Sign::Negative,
            Some(_) => Sign::Positive,
            None => self.side,
        }
    }

    fn flat(&self, powerline: &Powerline) -> bool {
        powerline.filled(self.side) == " "
    }

    fn divider_lead(&self, dir: Sign) -> String {
        let fill_flat = self.flat(self.fill);
        let divider_flat = self.flat(self.divider);
        if fill_flat != divider_flat {
            String::from("\u{2502} ")
        } else if divider_flat {
            self.divider.divider(dir).to_string()
        } else {
            format!("{} ", self.divider.divider(dir))
        }
    }
}
