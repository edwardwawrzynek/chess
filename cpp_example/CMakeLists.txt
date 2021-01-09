# If you add a new cpp file, add it here
# ex: set(SOURCES example.cpp file1.cpp file2.cpp file3.cpp)
set(SOURCES example.cpp)

# Compile directly with library files so we can use lto
set(CLIB_SOURCES ../lib/src/bitboard.c ../lib/src/move_gen.c ../lib/src/found_magics.c)
set(CPPLIB_SOURCES ../cpp_binding/chess.cpp)

add_executable(cpp_example ${SOURCES} ${CLIB_SOURCES} ${CPPLIB_SOURCES})

# If you don't need lto:
# add_executable(cpp_example ${SOURCES})
# target_link_libraries(cpp_example chess)

target_link_libraries(cpp_example pthread)
target_include_directories(cpp_example PRIVATE ../cpp_binding/ ../lib/include .)
