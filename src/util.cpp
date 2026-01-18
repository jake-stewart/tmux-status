#include "../include/util.hpp"
#include <cstring>

std::string execGetline(std::string cmd) {
    FILE *fp;
    char buf[256];
    std::string result = "";
    fp = popen(cmd.c_str(), "r");
    if (fp == NULL) {
        return result;
    }
    if (fgets(buf, sizeof(buf), fp) != NULL) {
        buf[strcspn(buf, "\n")] = 0;
        result = buf;
    }
    pclose(fp);
    return result;
}

std::string getTime(const char *fmt) {
    time_t rawtime;
    struct tm * timeinfo;
    char buffer[80];
    time(&rawtime);
    timeinfo = localtime(&rawtime);
    strftime(buffer, sizeof(buffer), fmt, timeinfo);
    return buffer;
}

int display_width(const char *s) {
    int width = 0;
    while (*s) {
        wchar_t wc;
        int len = mbtowc(&wc, s, MB_CUR_MAX);
        if (len <= 0) {
            break;
        }
        width += wcwidth(wc);
        s += len;
    }
    return width;
}
