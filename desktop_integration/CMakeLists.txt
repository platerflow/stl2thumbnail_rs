cmake_minimum_required(VERSION 3.1)

include(GNUInstallDirs)

project(stl2thumbnail-desktop VERSION 1.0)

set(RUST_BUILD_DIR ${CMAKE_SOURCE_DIR}/../target/release)
set(RUST_CLI_PATH ${RUST_BUILD_DIR}/stl2thumbnail)
set(RUST_INC_DIR ${CMAKE_SOURCE_DIR}/../include)

# stl2thumbnail rust lib
add_library(stl2thumbnail-rs STATIC IMPORTED)
set_target_properties(stl2thumbnail-rs PROPERTIES
    IMPORTED_LOCATION "${RUST_BUILD_DIR}/libstl2thumbnail.a"
    IMPORTED_LINK_INTERFACE_LIBRARIES "dl;m;pthread"
)

add_executable(stl2thumbnail-cli IMPORTED)
set_target_properties(stl2thumbnail-cli PROPERTIES
    IMPORTED_LOCATION "${RUST_CLI_PATH}"
)

link_directories(${RUST_BUILD_DIR})
include_directories(${RUST_INC_DIR})

add_subdirectory(kde)
add_subdirectory(gnome)

# install CLI
install(PROGRAMS ${RUST_CLI_PATH} DESTINATION ${CMAKE_INSTALL_BINDIR})
