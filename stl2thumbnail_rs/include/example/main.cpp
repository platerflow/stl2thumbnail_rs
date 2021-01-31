#include "stl2thumbnail.h"
#include <stdio.h>

int main() {
    s2t::RenderSettings settings;
    settings.width = 256;
    settings.height = 256;
    settings.size_hint = false;
    settings.timeout = 0;
    
    s2t::PictureBuffer buffer = s2t::render("/path/to/stlfile.stl", settings);
    printf("ptr %p, len %i\n", (void*)buffer.data, buffer.len);
    s2t::free_picture_buffer(buffer);
    return 0;
}
