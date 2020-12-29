add_executable(cpp_example example.cpp)

target_link_libraries(cpp_example chess pthread)
target_include_directories(cpp_example PRIVATE ../cpp_binding/)
