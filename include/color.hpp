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

const char* getCurrentFg();
const char* getCurrentBg();
const char* getDefaultFg();
const char* getDefaultBg();

void setCurrentFg(const char *fg);
void setCurrentBg(const char *bg);
void setDefaultFg(const char *bg);
void setDefaultBg(const char *bg);
