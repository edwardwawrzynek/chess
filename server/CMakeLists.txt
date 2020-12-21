add_executable(server server.cpp)
target_link_libraries(server chess-util usockets z)
target_include_directories(server PRIVATE ../lib/include .)

