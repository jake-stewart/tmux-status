#include "../include/block.hpp"
#include "../include/color.hpp"
#include "../include/util.hpp"

BlockSpan& BlockSpan::bold() {
    m_attr = "bold";
    return *this;
}

BlockSpan& BlockSpan::reverse() {
    m_attr = "reverse";
    return *this;
}

BlockSpan& BlockSpan::fg(const char *fg) {
    m_fg = fg;
    return *this;
}

BlockSpan& BlockSpan::bg(const char *bg) {
    m_bg = bg;
    return *this;
}

BlockSpan::BlockSpan(std::string text) {
    m_text = text;
    m_size = display_width(m_text.c_str());
}

void BlockSpan::print() const {
    std::string open = "";
    std::string close = "";

    if (getCurrentFg() == m_fg && getCurrentBg() == m_bg) {
        if (m_attr) {
            open = "#[" + std::string(m_attr) + "]";
            close = "#[no" + std::string(m_attr) + "]";
        }
    }
    else {
        open = "#[";
        if (getCurrentFg() != m_fg) {
            open += "fg=" + std::string(m_fg);
            if (getCurrentBg() != m_bg || m_attr) {
                open += ",";
            }
        }
        if (getCurrentBg() != m_bg) {
            open += "bg=" + std::string(m_bg);
            if (m_attr) {
                open += ",";
            }
        }
        if (m_attr) {
            open += m_attr;
            close = "#[no" + std::string(m_attr) + "]";
        }
        open += "]";
    }

    std::string output = open + m_text + close;
    printf("%s", output.c_str());

    setCurrentFg(m_fg);
    setCurrentBg(m_bg);
}

int BlockSpan::length() const {
    return m_size;
}

void NO_OP(const Block *block) {
}

Block::Block() {
    this->m_onClick = NO_OP;
    this->m_onDrag = NO_OP;
}

void Block::add(BlockSpan span) {
    m_spans.push_back(span);
}

int Block::length() const {
    int total = 0;
    for (const BlockSpan& span : m_spans) {
        total += span.length();
    }
    return total;
}

void Block::print() const {
    for (const BlockSpan& span : m_spans) {
        span.print();
    }
}

void Block::onClick(std::function<void (const Block *block)> callback) {
    m_onClick = callback;
}

void Block::click() const {
    m_onClick(this);
}

void Block::onDrag(std::function<void (const Block *block)> callback) {
    m_onDrag = callback;
}

void Block::drag() const {
    m_onDrag(this);
}
