add_executable(chess_server server.cpp)
target_link_libraries(chess_server chess-util usockets z)
target_include_directories(chess_server PRIVATE ../lib/include .)

install(TARGETS chess_server DESTINATION ${CMAKE_INSTALL_BINDIR})

