#include "../include/color.hpp"

const char *current_fg = DEFAULT;
const char *current_bg = DEFAULT;
const char *default_fg = DEFAULT;
const char *default_bg = DEFAULT;

void setCurrentFg(const char *fg) {
    current_fg = fg;
}

void setCurrentBg(const char *bg) {
    current_bg = bg;
}

void setDefaultFg(const char *fg) {
    default_fg = fg;
}

void setDefaultBg(const char *bg) {
    default_bg = bg;
}

const char* getCurrentFg() {
    return current_fg;
}

const char* getCurrentBg() {
    return current_bg;
}

const char* getDefaultFg() {
    return default_fg;
}

const char* getDefaultBg() {
    return default_bg;
}
