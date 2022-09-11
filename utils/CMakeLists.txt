add_executable(find_magics find_magics.c)

add_executable(perft_divide perft_divide.c)
target_link_libraries(perft_divide chess-util)
target_include_directories(perft_divide PRIVATE ../lib/include)

add_executable(perft_test perft_test.c)
target_link_libraries(perft_test chess-util)
target_include_directories(perft_test PRIVATE ../lib/include)

