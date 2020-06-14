#pragma once

#include <stdint.h>
#include <stddef.h>

// Linking to libstl2thumbnail requires -ldl -lm -pthread

typedef struct PictureBuffer {
    const uint8_t* data;
    size_t len;
    size_t stride;
    size_t depth;
} PictureBuffer;

PictureBuffer s2t_render(const char* path, size_t width, size_t height);
void s2t_free_picture_buffer(PictureBuffer buffer);
