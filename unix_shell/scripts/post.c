#include <stdio.h>

int main() {
    char lastchar = 0;
    char cur_char = 0;
    while (1) {
        cur_char = getchar();
        if (cur_char == EOF)
            break;
        putchar(cur_char);
        lastchar = cur_char;
    }
    if (lastchar != '\n') {
        putchar('\n');
    }
}
