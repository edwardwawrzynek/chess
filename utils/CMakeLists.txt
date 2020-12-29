add_executable(find_magics find_magics.c)

add_executable(perft_divide perft_divide.c)
target_link_libraries(perft_divide chess-util)
target_include_directories(perft_divide PRIVATE ../lib/include)

add_executable(perft_test perft_test.c)
target_link_libraries(perft_test chess-util)
target_include_directories(perft_test PRIVATE ../lib/include)

add_executable(sample sample.c sample_game.c)
target_link_libraries(sample chess-util wsclient)
target_include_directories(sample PRIVATE ../lib/include)

check_ipo_supported(RESULT supported OUTPUT error)
if (supported)
    message(STATUS "IPO / LTO supported")
    set_property(TARGET perft_test PROPERTY INTERPROCEDURAL_OPTIMIZATION TRUE)
    set_property(TARGET perft_divide PROPERTY INTERPROCEDURAL_OPTIMIZATION TRUE)
else ()
    message(STATUS "IPO / LTO not supported: <${error}>")
endif ()
