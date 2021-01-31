#include "stl2thumbnail.h"
#include <stdio.h>

int main() {
    PictureBuffer buffer = s2t_render("/path/to/stlfile.stl", 256, 256);
    printf("ptr %p, len %i\n", (void*)buffer.data, buffer.len);
    s2t_free_picture_buffer(buffer);
    return 0;
}
