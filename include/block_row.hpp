#include <vector>
#include "block.hpp"

struct BlockRow
{
    std::vector<Block> blocks;
    BlockRow(std::vector<Block> blocks);
    BlockRow();
    bool click(int pos) const;
    bool drag(int pos) const;
    int length() const;
    void clear();
    void shorten(int target, bool from_back);
    void removeEmpty();
    void print() const;
    const Block *at(int pos) const;
    int positionOf(const Block *block) const;
    int indexOf(const Block *block) const;
};

int resize(BlockRow& left_row, BlockRow& right_row, int width);
