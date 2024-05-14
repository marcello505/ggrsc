# Copyright (C) 2020-2022 Micah Snyder.

cmake_minimum_required(VERSION 3.18)
set(RUSTC_MINIMUM_REQUIRED 1.56)

project(ggrs-cpp
        VERSION "0.1.0"
        DESCRIPTION "A demo app to show a CMake project with components written in Rust.")

set(CMAKE_MODULE_PATH "${CMAKE_CURRENT_SOURCE_DIR}/cmake" ${CMAKE_MODULE_PATH})

#
# Find Build Tools
#
set(MAINTAINER_MODE_DEFAULT OFF)
option(MAINTAINER_MODE
    "Use `cbindgen` to generate Rust library API headers."
    ${MAINTAINER_MODE_DEFAULT})

find_package(Rust REQUIRED)

# Always use '-fPIC'/'-fPIE' option.
set(CMAKE_POSITION_INDEPENDENT_CODE ON)

# Include GNUInstallDirs for access to CMAKE_INSTALL_LIBDIR, etc
include(GNUInstallDirs)

# Enable CTest
if(CMAKE_PROJECT_NAME STREQUAL PROJECT_NAME)
    include(CTest)
    enable_testing()
endif()

# Enable source packages with CTest
set(CPACK_SOURCE_GENERATOR "TGZ")
set(CPACK_SOURCE_PACKAGE_FILE_NAME cmake-rust-${PROJECT_VERSION})
set(CPACK_SOURCE_IGNORE_FILES
    \\.git/
    build/
    ".*~$"
)
set(CPACK_VERBATIM_VARIABLES YES)
include(CPack)

#
# Build targets.
#
add_rust_library(TARGET ggrs_cpp
    SOURCE_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}"
    BINARY_DIRECTORY "${CMAKE_BINARY_DIR}"
)
install(FILES $<TARGET_FILE:ggrs_cpp> DESTINATION ${CMAKE_INSTALL_LIBDIR} COMPONENT libraries)
add_library(ggrs-cpp ALIAS ggrs_cpp)


#
# The Summary Info.
#
string(TOUPPER "${CMAKE_BUILD_TYPE}" _build_type)
message(STATUS "Configuration Options Summary --
    Target system:          ${CMAKE_SYSTEM}
    Compiler:
        Build type:         ${CMAKE_BUILD_TYPE}
        C compiler:         ${CMAKE_C_COMPILER}
        Rust toolchain:     ${cargo_EXECUTABLE} (${cargo_VERSION})
                            ${rustc_EXECUTABLE} (${rustc_VERSION})
        CFLAGS:             ${CMAKE_C_FLAGS_${_build_type}} ${CMAKE_C_FLAGS}
    Build Options:
        Maintainer Mode:    ${MAINTAINER_MODE}")

