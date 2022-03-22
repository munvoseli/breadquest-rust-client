#include <stdio.h>
#include <string.h>

int main(int argc, char** argv) {
	int counts[256];
	memset(counts, 0, 256 * sizeof(int));
	for (int i = 1; i < argc; ++i) {
//		printf("sdk %d\n", i);
		FILE* fp = fopen(argv[i], "r");
		int c;
		for (int k = 0; k < 9; ++k) fgetc(fp);
		while ((c = fgetc(fp)) != EOF) {
			if (c < 0 || c >= 256)
				printf("ahhdsa\n");
			counts[c]++;
			if (c == 0x94)
				printf("%s\n", argv[i]);
		}
		fclose(fp);
	}
	for (int i = 0; i < 256; ++i) {
		printf("%x %d\n", i, counts[i]);
	}
	return 0;
}
