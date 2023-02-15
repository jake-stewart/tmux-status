#include <string>
#include <vector>
#include <filesystem>

#include "../include/color.hpp"
#include "../include/block_row.hpp"
#include "../include/util.hpp"

using std::string;
namespace fs = std::filesystem;

string pane_title;
fs::path pane_path;
string window_str;
int window_idx;
string session_title;
int client_width;
bool is_zoomed;
int mouse_x;
const char *color = GREEN;

string getProjectDetails(string& session_name, fs::path& pane_path) {
    const string sed = "sed 's+^refs/heads/+:+'";
    const string git = "git symbolic-ref HEAD 2>/dev/null";
    const string cmd
        = "(cd '" + pane_path.string() + "' && " + git + " | " + sed + ")";

    return session_name + execGetline(cmd);
}


std::vector<Block*> createTabBlocks(int active_idx, string& window_str) {
    std::vector<Block*> blocks;

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
            Block *block = new Block();
            if (idx == active_idx) {
                block->add(BlockSpan(" " + x + " ").bold().fg(GREY_0).bg(color));
            }
            else {
                std::string content;
                if (active_idx != idx && active_idx != idx - 1 && idx) {
                    if (idx < active_idx) {
                        block->add(BlockSpan(" ▏").fg(GREY_6));
                        block->add(BlockSpan(x));
                    }
                    else {
                        block->add(BlockSpan("▉").fg(GREY_6).reverse());
                        block->add(BlockSpan(" " + x));
                    }
                }
                else {
                    block->add(BlockSpan(" " + x));
                }
                if (active_idx == idx + 1) {
                    block->add(BlockSpan(" "));
                }
            }
            block->onClick([idx]() {
                system(string(("tmux select-window -t ") + std::to_string(idx)).c_str());
            });
            blocks.push_back(block);
            start = ++end;
        }
        idx++;
    }

    return blocks;
}

BlockRow createLeftRow() {
    std::vector<Block*> left_blocks = createTabBlocks(window_idx, window_str);
    BlockRow left_row(left_blocks);
    left_row.removeEmpty();
    return left_row;
}

BlockRow createRightRow() {
    std::vector<Block*> right_blocks;

    // pane title
    Block *block = new Block();
    block->add(BlockSpan(pane_title));
    block->add(BlockSpan(" ▏").fg(GREY_6));
    right_blocks.push_back(block);

    // cwd
    block = new Block();
    std::string filename;
    filename = pane_path == "/" ? "/" : pane_path.filename().string();
    if (filename.length()) {
        block->add(BlockSpan(filename + " "));
    }
    else {
        block->add(BlockSpan("unknown working directory ").fg(RED));
    }
    right_blocks.push_back(block);

    // project
    block = new Block();
    block->add(BlockSpan(" " + getProjectDetails(session_title,
                    pane_path) + " ").fg(GREY_0).bg(color).bold());
    block->onClick([] () {
        system("~/.config/tmux/popup-switch-session.sh");
    });
    right_blocks.push_back(block);

    // date
    block = new Block();
    block->add(BlockSpan(" " + getTime("%H:%M %d-%b-%y ")));
    right_blocks.push_back(block);

    BlockRow right_row(right_blocks);
    right_row.removeEmpty();

    return right_row;
}


int main(int argc, const char **argv) {
    pane_title    = argv[1];
    pane_path     = argv[2];
    window_str    = argv[3];
    window_idx    = std::stoi(argv[4]);
    session_title = argv[5];
    client_width  = std::stoi(argv[6]);
    is_zoomed     = std::stoi(argv[7]);
    mouse_x       = std::stoi(argv[8]);

    if (std::string(session_title) == "popup") {
        return 0;
    }

    std::string colorEnv = execGetline(
        "tmux show-environment session_color | sed 's/.*=//'"
    );

    if (colorEnv.size()) {
        color = colorEnv.c_str();
    }
    else {
        color = session_title == "main" ? "cyan" : "brightblack";
    }

    BlockRow left_row = createLeftRow();
    BlockRow right_row = createRightRow();

    int remaining = resize(left_row, right_row, client_width);

    if (mouse_x >= 0) {
        if (!left_row.click(mouse_x)) {
            right_row.click(mouse_x - (client_width - right_row.length()));
        }
        return 0;
    }


    printf("#[bg=%s]", DEFAULT_BG);
    setCurrentBg(DEFAULT_BG);

    left_row.print();

    printf("#[bg=%s]%*s", DEFAULT_BG, remaining, "");
    setCurrentBg(DEFAULT_BG);

    right_row.print();

    return 0;
}
