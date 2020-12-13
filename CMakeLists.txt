cmake_minimum_required(VERSION 3.17)
project(chess)

set(CMAKE_C_STANDARD 11)

set(CFLAGS -Wall -Wextra -pedantic -Wno-unused-parameter -O3)

set(SOURCES src/bitboard.c src/move_gen.c src/found_magics.c)
set(DRIVER src/main.c)
set(FIND_MAGICS src/find_magics.c)

include_directories(src)

add_executable(chess ${SOURCES} ${DRIVER})
add_executable(chess_find_magics ${SOURCES} ${FIND_MAGICS})

target_compile_options(chess PRIVATE ${CFLAGS})
target_compile_options(chess_find_magics PRIVATE ${CFLAGS})

include(CheckIPOSupported)
check_ipo_supported(RESULT supported OUTPUT error)

if( supported )
    message(STATUS "IPO / LTO supported")
    set_property(TARGET chess PROPERTY INTERPROCEDURAL_OPTIMIZATION TRUE)
else()
    message(STATUS "IPO / LTO not supported: <${error}>")
endif()

