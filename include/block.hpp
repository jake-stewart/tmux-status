#include <string>
#include <functional>
#include <vector>

#include "color.hpp"

class BlockSpan {
    private:
        std::string m_text;
        int m_size;
        const char *m_attr = nullptr;
        const char *m_fg = DEFAULT_FG;
        const char *m_bg = DEFAULT_BG;

    public:
        BlockSpan& bold();
        BlockSpan& reverse();
        BlockSpan& fg(const char *fg);
        BlockSpan& bg(const char *bg);
        BlockSpan(std::string text);
        void print();
        int length();
};

class Block
{
    private:
        std::vector<BlockSpan> m_spans;
        std::function<void (void)> m_onClick;

    public:
        Block();
        void add(BlockSpan span);
        int length();
        void print();
        void onClick(std::function<void (void)> callback);
        void click();
};

