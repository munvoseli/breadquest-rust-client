#include <stdio.h>
#include <string.h>

char* hex = "0123456789abcdef";

int main(int argc, char** argv) {
	for (int i = 1; i < argc; ++i) {
		FILE* fp = fopen(argv[i], "r");
		int j = 0;
		int c;
		for (int k = 0; k < 9; ++k) fgetc(fp);
		char show = 0;
		while ((c = fgetc(fp)) != EOF) {
			if (c == 0x96 || c == 0x95) {
				show = 1;
				break;
			}
		}
		if (show == 0) goto no;
		rewind(fp);
		for (int k = 0; k < 9; ++k) fgetc(fp);
		while ((c = fgetc(fp)) != EOF) {
//			putchar(hex[c >> 4]);
//			putchar(hex[c & 15]);
			if (c < 2) putchar('-');
			else if (c == 2) putchar('?');
			else if (c <= 0x20) putchar('?');
			else if (c <= 0x7f) putchar(c);
			else if (c == 0x80) putchar(' ');
			else if (c <= 0x88) putchar('#');
			else if (c <= 0x90) putchar('.');
			else if (c <= 0x93) putchar('n');
			else if (c == 0x94) putchar('b');
			else if (c <= 0x96) putchar('!');
			else putchar('h');
			++j;
			if (j % 128 == 0) putchar(10);
		}
		putchar(10);
		putchar(10);
	no:	fclose(fp);
	}
	return 0;
}
