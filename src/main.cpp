#include <string>
#include <vector>
#include <filesystem>
#include <locale.h>

#include "../include/color.hpp"
#include "../include/block_row.hpp"
#include "../include/util.hpp"

using std::string;
namespace fs = std::filesystem;

string pane_title;
fs::path pane_path;
string window_str;
int window_idx;
string session_id;
string session_color;
string session_title;
int client_width;
int selection_y_start = -1;
int selection_y_end = -1;
int selection_x_start = -1;
int selection_x_end = -1;
bool is_zoomed;
int mouse_x;
int mouse_mode;

string color = GREEN;

string getBranch(fs::path& pane_path) {
    const string git = "git rev-parse --abbrev-ref HEAD 2>/dev/null";
    const string branch = execGetline("(cd '" + pane_path.string() + "' && " + git + ")");
    return branch;
}

int rgb256(int r, int g, int b) {
    return 16 + r * 36 + g * 6 + b;
}

BlockRow createTabBlocks(int active_idx, string& window_str) {
    BlockRow row;

    int len = window_str.size();

    int idx = 0;
    int start = 0;
    int end = 0;
    while (start < len) {
        do {
            end++;
        }
        while (end < len && window_str[end] != ',');

        if (start != end) {
            std::string x = window_str.substr(start, end - start);
            Block block;
            if (idx == active_idx) {
                block.add(BlockSpan(" " + x + " ").bold().fg(GREY_0).bg(color.c_str()));
            }
            else {
                std::string content;
                if (active_idx != idx && active_idx != idx - 1 && idx) {
                    if (idx < active_idx) {
                        block.add(BlockSpan("▏").fg(GREY_6));
                        block.add(BlockSpan(x + " "));
                    }
                    else {
                        block.add(BlockSpan("▏").fg(GREY_6));
                        block.add(BlockSpan(x + " "));
                    }
                }
                else {
                    block.add(BlockSpan(" " + x + " "));
                }
            }
            if (idx != active_idx) {
                block.onClick([idx](const Block *clicked) {
                    system(string(("tmux select-window -t ")
                              + std::to_string(idx)).c_str());
                });
                block.onDrag([&row, active_idx](const Block *dragged) {
                    if (active_idx >= row.blocks.size() || active_idx < 0) {
                        return;
                    }
                    int dst = row.positionOf(dragged);
                    int src = row.positionOf(&row.blocks[active_idx]);
                    int draggedIdx = row.indexOf(dragged);
                    if (src == -1 || dst == -1 || draggedIdx == -1) {
                        return;
                    }
                    if (dst < src) {
                        if (mouse_x < src + row.blocks[active_idx].length() - dragged->length()) {
                            system(string(("tmux move-window -b -t ")
                                  + std::to_string(draggedIdx)).c_str());
                        }
                    }
                    else {
                        if (mouse_x >= src + dragged->length()) {
                            system(string(("tmux move-window -a -t ")
                                  + std::to_string(draggedIdx)).c_str());
                        }
                    }
                });
            }
            row.blocks.push_back(block);
            start = ++end;
        }
        idx++;
    }

    row.removeEmpty();
    return row;
}

BlockRow createRightRow() {
    BlockRow row;

    // pane title
    Block block;
    block.add(BlockSpan(pane_title));
    block.add(BlockSpan(" ▏").fg(GREY_6));
    row.blocks.push_back(block);

    const string branch = getBranch(pane_path);

    // cwd
    block = Block();
    std::string filename;
    filename = pane_path == "/" ? "/" : pane_path.filename().string();
    if (filename.length()) {
        // block->add(BlockSpan(filename + " \uf115  "));
        // block->add(BlockSpan("\uf115  " + filename + " "));
        block.add(BlockSpan(filename));
    }
    else {
        block.add(BlockSpan("???").fg(RED));
    }
    if (branch.length() == 0) {
        block.add(BlockSpan(" "));
    }
    row.blocks.push_back(block);

    block = Block();
    if (branch.length() > 0) {
        block.add(BlockSpan(" ▏").fg(GREY_6));
        block.add(BlockSpan("\ue0a0 " + branch + " "));
    }
    row.blocks.push_back(block);

    // project
    block = Block();
    if (selection_y_start != selection_y_end) {
        int maxY = std::max(selection_y_start, selection_y_end);
        int minY = std::min(selection_y_start, selection_y_end);
        session_title = std::to_string(maxY - minY + 1) + " rows";
        block.add(BlockSpan(" " + session_title + " ").fg(GREY_0).bg(color.c_str()).bold());
    }
    else if (selection_x_start != -1) {
        int maxX = std::max(selection_x_start, selection_x_end);
        int minX = std::min(selection_x_start, selection_x_end);
        int num = maxX - minX + 1;
        session_title = std::to_string(num) + " col" + (num == 1 ? "" : "s");
        block.add(BlockSpan(" " + session_title + " ").fg(GREY_0).bg(color.c_str()).bold());
    }
    else {
        block.add(BlockSpan(" " + session_title + " ").fg(GREY_0).bg(color.c_str()).bold());
        block.onClick([] (const Block *clicked) {
            system("~/.config/tmux/popup-switch-session.sh");
        });
    }
    row.blocks.push_back(block);

    // date
    block = Block();
    block.add(BlockSpan(" " + getTime("%H:%M %d-%b-%y ")));
    block.onClick([](const Block *clicked) {
        system("~/.config/tmux/popup-cal.sh");
    });
    row.blocks.push_back(block);

    BlockRow right_row(row.blocks);
    right_row.removeEmpty();

    return row;
}


int main(int argc, const char **argv) {
    setlocale(LC_ALL, "");
    pane_title = argv[1];
    pane_path = argv[2];
    window_str = argv[3];
    window_idx = std::stoi(argv[4]);
    session_id = argv[5];
    session_title = argv[6];
    session_color = argv[7];
    client_width = std::stoi(argv[8]);
    if (strlen(argv[9])) {
        selection_y_start = std::stoi(argv[9]);
        selection_y_end = std::stoi(argv[10]);
        selection_x_start = std::stoi(argv[11]);
        selection_x_end = std::stoi(argv[12]);
    }
    is_zoomed = std::stoi(argv[13]);
    mouse_mode = std::stoi(argv[14]);
    mouse_x = std::stoi(argv[15]);

    setDefaultBg(is_zoomed ? GREY_5 : GREY_2);

    if (std::string(session_title) == "popup") {
        return 0;
    }

    std::string themedColor = "";

    color = session_title == "scratch" ? "cyan" : "brightblack";
    int n = 3;
    if (session_color == "red") {
        color = "colour" + std::to_string(rgb256(3, 0, 0));
    }
    else if (session_color == "green") {
        color = "colour" + std::to_string(rgb256(0, 3, 0));
    }
    else if (session_color == "blue") {
        color = "colour" + std::to_string(rgb256(0, 1, 4));
    }
    else if (session_color == "yellow") {
        color = "colour" + std::to_string(rgb256(3, 3, 0));
    }
    else if (session_color == "cyan") {
        color = "colour" + std::to_string(rgb256(0, 3, 3));
    }
    else if (session_color == "magenta" || session_color == "purple") {
        color = "colour" + std::to_string(rgb256(3, 0, 3));
    }
    else if (session_title == "scratch") {
        color = "cyan";
    }
    else {
        color = "brightblack";
    }

    BlockRow left_row = createTabBlocks(window_idx, window_str);
    left_row.removeEmpty();

    BlockRow right_row = createRightRow();

    int remaining = resize(left_row, right_row, client_width);

    if (mouse_mode == 1) {
        if (!left_row.click(mouse_x)) {
            right_row.click(mouse_x - (client_width - right_row.length()));
        }
        return 0;
    }
    else if (mouse_mode == 2) {
        if (!left_row.drag(mouse_x)) {
            right_row.drag(mouse_x - (client_width - right_row.length()));
        }
        return 0;
    }

    printf("#[bg=%s]", getDefaultBg());
    setCurrentBg(getDefaultBg());

    left_row.print();

    printf("#[bg=%s]%*s", getDefaultBg(), remaining, "");
    setCurrentBg(getDefaultBg());

    right_row.print();

    return 0;
}
