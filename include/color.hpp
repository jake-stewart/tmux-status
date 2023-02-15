#define DEFAULT "default"
#define GREEN "green"
#define GREY "brightblack"
#define WHITE "white"
#define BLACK "black"
#define RED "red"

#define GREY_0 "colour232"
#define GREY_1 "colour233"
#define GREY_2 "colour234"
#define GREY_3 "colour235"
#define GREY_4 "colour236"
#define GREY_5 "colour237"
#define GREY_6 "colour238"

#define DEFAULT_BG GREY_2
#define DEFAULT_FG DEFAULT

const char* getCurrentFg();
const char* getCurrentBg();
void setCurrentFg(const char *fg);
void setCurrentBg(const char *bg);
