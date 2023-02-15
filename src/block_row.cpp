#include "../include/block_row.hpp"
#include "../include/color.hpp"

BlockRow::BlockRow(std::vector<Block*>& blocks) {
    m_blocks = blocks;
}

bool BlockRow::click(int pos) {
    int total = 0;
    for (Block *block : m_blocks) {
        if (pos >= total && pos < total + block->length()) {
            block->click();
            return true;
        }
        total += block->length();
    }
    return false;
}

int BlockRow::length() {
    int total = 0;
    for (int i = 0; i < m_blocks.size(); i++) {
        total += m_blocks[i]->length();
    }
    return total;
}

void BlockRow::clear() {
    m_blocks.clear();
}

void BlockRow::shorten(int target, bool from_back) {
    while (m_blocks.size() && length() > target) {
        if (from_back) {
            m_blocks.pop_back();
        }
        else {
            m_blocks.erase(m_blocks.begin());
        }
    }
}

void BlockRow::removeEmpty() {
    int offset = 0;
    for (int i = 0; i < m_blocks.size(); i++) {
        if (!m_blocks[i]->length()) {
            offset++;
        }
        else {
            m_blocks[i - offset] = m_blocks[i];
        }
    }
    m_blocks.resize(m_blocks.size() - offset);
}

void BlockRow::print() {
    for (int i = 0; i < m_blocks.size(); i++) {
        m_blocks[i]->print();
    }
}

int resize(BlockRow& left_row, BlockRow& right_row, int width) {
    int remaining = width - left_row.length();
    if (remaining <= 0) {
        right_row.clear();
        left_row.shorten(width, true);
        remaining = 0;
    }
    else {
        remaining -= right_row.length();
        if (remaining < 3) {
            right_row.shorten(width - left_row.length() - 3, false);
            remaining = width - left_row.length() - right_row.length();
        }
    }
    return remaining;
}
