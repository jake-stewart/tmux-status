#include <vector>
#include "block.hpp"

class BlockRow
{
private:
    std::vector<Block*> m_blocks;

public:
    BlockRow(std::vector<Block*>& blocks);
    bool click(int pos);
    int length();
    void clear();
    void shorten(int target, bool from_back);
    void removeEmpty();
    void print();
};

int resize(BlockRow& left_row, BlockRow& right_row, int width);
