use std::rc::Rc;

use tuie::prelude::*;

pub const TRUNCATE: &str = "\u{2026}";

enum End {
    Lead,
    Trail,
}

#[derive(Clone)]
pub struct Block {
    spans: Vec<(String, Style)>,
    on_click: Option<Rc<dyn Fn()>>,
    tag: Option<&'static str>,
    fill: Option<Style>,
    flex: Option<Place>,
    truncate: bool,
}

impl Block {
    pub fn new() -> Self {
        Self {
            spans: Vec::new(),
            on_click: None,
            tag: None,
            fill: None,
            flex: None,
            truncate: false,
        }
    }

    pub fn span(mut self, text: impl Into<String>, style: Style) -> Self {
        self.spans.push((text.into(), style));
        self
    }

    pub fn prepend(mut self, text: impl Into<String>, style: Style) -> Self {
        self.spans.insert(0, (text.into(), style));
        self
    }

    pub fn uncap(&mut self) {
        if !self.spans.is_empty() {
            self.spans.remove(0);
        }
    }

    pub fn cap_leading(&mut self, glyph: &str, bar_bg: Color) {
        self.pad_leading();
        if glyph != " "
            && let Some(bg) = self.leading_bg()
        {
            self.spans
                .insert(0, (glyph.to_string(), Style::new().fg(bg).bg(bar_bg)));
        }
    }

    pub fn cap_trailing(&mut self, glyph: &str, bar_bg: Color) {
        self.pad_trailing();
        if glyph != " "
            && let Some(bg) = self.trailing_bg()
        {
            self.spans
                .push((glyph.to_string(), Style::new().fg(bg).bg(bar_bg)));
        }
    }

    pub fn pad_trailing(&mut self) {
        if let Some(bg) = self.trailing_bg() {
            self.spans.push((String::from(" "), Style::new().bg(bg)));
        }
    }

    pub fn pad_leading(&mut self) {
        if let Some(bg) = self.leading_bg() {
            self.spans.insert(0, (String::from(" "), Style::new().bg(bg)));
        }
    }

    pub fn on_click(mut self, callback: impl Fn() + 'static) -> Self {
        self.on_click = Some(Rc::new(callback));
        self
    }

    pub fn tag(mut self, tag: &'static str) -> Self {
        self.tag = Some(tag);
        self
    }

    pub fn fill(mut self, fill: Style) -> Self {
        self.fill = Some(fill);
        self
    }

    pub fn flex(mut self, place: Place) -> Self {
        self.flex = Some(place);
        self
    }

    pub fn truncate(mut self) -> Self {
        self.truncate = true;
        self
    }

    pub fn filled(&self) -> bool {
        self.fill.is_some()
    }

    pub fn flexed(&self) -> bool {
        self.flex.is_some()
    }

    pub fn truncatable(&self) -> bool {
        self.truncate
    }

    pub fn truncate_to(&mut self, max: usize) {
        if self.width() <= max {
            return;
        }
        let budget = max.saturating_sub(tuie::display_width(TRUNCATE));
        let mut used = 0;
        let mut spans = Vec::new();
        for (text, style) in std::mem::take(&mut self.spans) {
            let w = tuie::display_width(&text);
            if used + w <= budget {
                used += w;
                spans.push((text, style));
                continue;
            }
            let kept = clip_width(&text, budget - used);
            spans.push((format!("{}{}", kept, TRUNCATE), style));
            break;
        }
        self.spans = spans;
    }

    pub fn leading_bg(&self) -> Option<Color> {
        self.spans.first().and_then(|(_, style)| style.get_bg())
    }

    pub fn trailing_bg(&self) -> Option<Color> {
        self.spans.last().and_then(|(_, style)| style.get_bg())
    }

    fn recolor(&mut self, end: End, f: impl Fn(Style) -> Style) {
        if self.spans.len() < 2 {
            return;
        }
        let span = match end {
            End::Lead => self.spans.first_mut(),
            End::Trail => self.spans.last_mut(),
        };
        if let Some((_, style)) = span {
            *style = f(*style);
        }
    }

    pub fn recolor_leading_bg(&mut self, bg: Color) {
        self.recolor(End::Lead, |style| style.bg(bg));
    }

    pub fn recolor_trailing_bg(&mut self, bg: Color) {
        self.recolor(End::Trail, |style| style.bg(bg));
    }

    pub fn recolor_leading_seam(&mut self, seam: Color, bg: Color) {
        self.recolor(End::Lead, |style| recolor_seam(style, seam, bg));
    }

    pub fn recolor_trailing_seam(&mut self, seam: Color, bg: Color) {
        self.recolor(End::Trail, |style| recolor_seam(style, seam, bg));
    }

    pub(crate) fn width(&self) -> usize {
        self.spans
            .iter()
            .map(|(text, _)| tuie::display_width(text))
            .sum()
    }

    pub fn is_empty(&self) -> bool {
        self.width() == 0
    }

    pub(crate) fn leading_width(&self) -> usize {
        self.spans
            .first()
            .map_or(0, |(text, _)| tuie::display_width(text))
    }

    #[cfg(test)]
    pub fn text(&self) -> String {
        self.spans.iter().map(|(text, _)| text.as_str()).collect()
    }

    pub(crate) fn into_parts(self) -> Parts {
        Parts {
            spans: self.spans,
            on_click: self.on_click,
            tag: self.tag,
            flex: self.flex,
        }
    }
}

pub(crate) struct Parts {
    pub spans: Vec<(String, Style)>,
    pub on_click: Option<Rc<dyn Fn()>>,
    pub tag: Option<&'static str>,
    pub flex: Option<Place>,
}

pub(crate) fn block_len(blocks: &[Block]) -> usize {
    blocks.iter().map(Block::width).sum()
}

fn clip_width(text: &str, max: usize) -> String {
    let mut out = String::new();
    let mut used = 0;
    for ch in text.chars() {
        let w = tuie::display_width(ch.encode_utf8(&mut [0; 4]));
        if used + w > max {
            break;
        }
        used += w;
        out.push(ch);
    }
    out
}

fn recolor_seam(style: Style, seam: Color, bg: Color) -> Style {
    if style.get_bg() == Some(seam) {
        style.bg(bg)
    } else {
        style.fg(bg)
    }
}
