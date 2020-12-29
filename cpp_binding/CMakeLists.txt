include(GNUInstallDirs)
set(SOURCES chess.cpp)

add_library(chess SHARED ${SOURCES})
set_target_properties(chess PROPERTIES VERSION 0.1 SOVERSION 1 PUBLIC_HEADER chess.hpp)
target_include_directories(chess PRIVATE ./ ../lib/include/)
target_link_libraries(chess chess-util)
install(TARGETS chess
        LIBRARY DESTINATION ${CMAKE_INSTALL_LIBDIR}
        PUBLIC_HEADER DESTINATION ${CMAKE_INSTALL_INCLUDEDIR})

check_ipo_supported(RESULT supported OUTPUT error)
if (supported)
    message(STATUS "IPO / LTO supported")
    set_property(TARGET chess PROPERTY INTERPROCEDURAL_OPTIMIZATION TRUE)
else ()
    message(STATUS "IPO / LTO not supported: <${error}>")
endif ()

