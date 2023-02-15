#include "../include/color.hpp"

const char *current_fg = DEFAULT;
const char *current_bg = DEFAULT;

void setCurrentFg(const char *fg) {
    current_fg = fg;
}

void setCurrentBg(const char *bg) {
    current_bg = bg;
}

const char* getCurrentFg() {
    return current_fg;
}

const char* getCurrentBg() {
    return current_bg;
}
