#include "stl2thumbnail.h"
#include <stdio.h>

int main() {
    PictureBuffer buffer = render("/path/to/stlfile.stl", 256, 256);
    printf("ptr %p, len %i\n", (void*)buffer.data, buffer.len);
    free_picture_buffer(buffer);
    return 0;
}
