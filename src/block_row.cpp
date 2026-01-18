#include "../include/block_row.hpp"
#include "../include/color.hpp"
#include "../include/util.hpp"

BlockRow::BlockRow() {}

BlockRow::BlockRow(std::vector<Block> blocks) {
    blocks = blocks;
}

bool BlockRow::click(int pos) const {
    int total = 0;
    for (const Block &block : blocks) {
        if (pos >= total && pos < total + block.length()) {
            block.click();
            return true;
        }
        total += block.length();
    }
    return false;
}

bool BlockRow::drag(int pos) const {
    int total = 0;
    for (const Block &block : blocks) {
        if (pos >= total && pos < total + block.length()) {
            block.drag();
            return true;
        }
        total += block.length();
    }
    return false;
}

int BlockRow::length() const {
    int total = 0;
    for (int i = 0; i < blocks.size(); i++) {
        total += blocks[i].length();
    }
    return total;
}

void BlockRow::clear() {
    blocks.clear();
}

void BlockRow::shorten(int target, bool from_back) {
    while (blocks.size() && length() > target) {
        if (from_back) {
            blocks.pop_back();
        }
        else {
            blocks.erase(blocks.begin());
        }
    }
}

void BlockRow::removeEmpty() {
    int offset = 0;
    for (int i = 0; i < blocks.size(); i++) {
        if (!blocks[i].length()) {
            offset++;
        }
        else {
            blocks[i - offset] = blocks[i];
        }
    }
    blocks.resize(blocks.size() - offset);
}

void BlockRow::print() const {
    for (int i = 0; i < blocks.size(); i++) {
        blocks[i].print();
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

const Block *BlockRow::at(int pos) const {
    int total = 0;
    for (int i = 0; i < blocks.size(); i++) {
        if (pos >= total && pos < total + blocks[i].length()) {
            return &blocks[i];
        }
        total += blocks[i].length();
    }
    return NULL;
}

int BlockRow::positionOf(const Block *block) const {
    int total = 0;
    for (int i = 0; i < blocks.size(); i++) {
        if (block == &blocks[i]) {
            return total;
        }
        total += blocks[i].length();
    }
    return -1;
}

int BlockRow::indexOf(const Block *block) const {
    for (int i = 0; i < blocks.size(); i++) {
        if (block == &blocks[i]) {
            return i;
        }
    }
    return -1;
}
