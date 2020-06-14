cmake_minimum_required(VERSION 3.1)

project(stl2thumbnail-desktop VERSION 1.0)

set(RUST_LIB_DIR ${CMAKE_SOURCE_DIR}/../target/release)
set(RUST_INC_DIR ${CMAKE_SOURCE_DIR}/../include)

# stl2thumbnail rust lib
add_library(stl2thumbnail-rs STATIC IMPORTED)
set_target_properties(stl2thumbnail-rs PROPERTIES
    IMPORTED_LOCATION "${RUST_LIB_DIR}/libstl2thumbnail.a"
    IMPORTED_LINK_INTERFACE_LIBRARIES "dl;m;pthread"
)

link_directories(${RUST_LIB_DIR})
include_directories(${RUST_INC_DIR})

add_subdirectory(kde)
