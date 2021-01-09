include(GNUInstallDirs)
set(SOURCES chess.cpp)

add_library(chess-cpp SHARED ${SOURCES})
set_target_properties(chess-cpp PROPERTIES VERSION 0.1 SOVERSION 1 PUBLIC_HEADER chess.hpp)
target_include_directories(chess-cpp PRIVATE ./ ../lib/include/)
target_link_libraries(chess-cpp chess-util)
install(TARGETS chess-cpp
        LIBRARY DESTINATION ${CMAKE_INSTALL_LIBDIR}
        COMPONENT chess-cpp
        PUBLIC_HEADER DESTINATION ${CMAKE_INSTALL_INCLUDEDIR}
        COMPONENT chess-cpp)

check_ipo_supported(RESULT supported OUTPUT error)
if (supported)
    message(STATUS "IPO / LTO supported")
    set_property(TARGET chess-cpp PROPERTY INTERPROCEDURAL_OPTIMIZATION TRUE)
else ()
    message(STATUS "IPO / LTO not supported: <${error}>")
endif ()

