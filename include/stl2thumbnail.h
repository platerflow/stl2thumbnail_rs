#pragma once

#include <stdint.h>
#include <stddef.h>

// Linking to libstl2thumbnail requires -ldl -lm -pthread

typedef struct s2t_PictureBuffer {
    const uint8_t* data;
    size_t len;
    size_t stride;
    size_t depth;
} PictureBuffer;

typedef struct s2t_Flags {
    bool size_hint;
} Flags;

s2t_PictureBuffer s2t_render(const char* path, size_t width, size_t height, Flags flags);
void s2t_free_picture_buffer(s2t_PictureBuffer buffer);
