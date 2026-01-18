#include <string>
#include <functional>
#include <vector>

#include "color.hpp"

class BlockSpan {
    private:
        std::string m_text;
        int m_size;
        const char *m_attr = nullptr;
        const char *m_fg = getDefaultFg();
        const char *m_bg = getDefaultBg();

    public:
        BlockSpan& bold();
        BlockSpan& reverse();
        BlockSpan& fg(const char *fg);
        BlockSpan& bg(const char *bg);
        BlockSpan(std::string text);
        void print() const;
        int length() const;
};

class Block
{
    private:
        std::vector<BlockSpan> m_spans;
        std::function<void (const Block *block)> m_onClick;
        std::function<void (const Block *block)> m_onDrag;

    public:
        Block();
        void add(BlockSpan span);
        int length() const;
        void print() const;
        void onClick(std::function<void (const Block *block)> callback);
        void onDrag(std::function<void (const Block *block)> callback);
        void click() const;
        void drag() const;
};

