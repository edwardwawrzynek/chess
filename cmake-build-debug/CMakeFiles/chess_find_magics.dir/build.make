# CMAKE generated file: DO NOT EDIT!
# Generated by "Unix Makefiles" Generator, CMake Version 3.17

# Delete rule output on recipe failure.
.DELETE_ON_ERROR:


#=============================================================================
# Special targets provided by cmake.

# Disable implicit rules so canonical targets will work.
.SUFFIXES:


# Disable VCS-based implicit rules.
% : %,v


# Disable VCS-based implicit rules.
% : RCS/%


# Disable VCS-based implicit rules.
% : RCS/%,v


# Disable VCS-based implicit rules.
% : SCCS/s.%


# Disable VCS-based implicit rules.
% : s.%


.SUFFIXES: .hpux_make_needs_suffix_list


# Command-line flag to silence nested $(MAKE).
$(VERBOSE)MAKESILENT = -s

# Suppress display of executed commands.
$(VERBOSE).SILENT:


# A target that is always out of date.
cmake_force:

.PHONY : cmake_force

#=============================================================================
# Set environment variables for the build.

# The shell in which to execute make rules.
SHELL = /bin/sh

# The CMake executable.
CMAKE_COMMAND = /opt/clion/bin/cmake/linux/bin/cmake

# The command to remove a file.
RM = /opt/clion/bin/cmake/linux/bin/cmake -E rm -f

# Escaping for special characters.
EQUALS = =

# The top-level source directory on which CMake was run.
CMAKE_SOURCE_DIR = /home/edward/Documents/chess

# The top-level build directory on which CMake was run.
CMAKE_BINARY_DIR = /home/edward/Documents/chess/cmake-build-debug

# Include any dependencies generated for this target.
include CMakeFiles/chess_find_magics.dir/depend.make

# Include the progress variables for this target.
include CMakeFiles/chess_find_magics.dir/progress.make

# Include the compile flags for this target's objects.
include CMakeFiles/chess_find_magics.dir/flags.make

CMakeFiles/chess_find_magics.dir/src/bitboard.c.o: CMakeFiles/chess_find_magics.dir/flags.make
CMakeFiles/chess_find_magics.dir/src/bitboard.c.o: ../src/bitboard.c
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green --progress-dir=/home/edward/Documents/chess/cmake-build-debug/CMakeFiles --progress-num=$(CMAKE_PROGRESS_1) "Building C object CMakeFiles/chess_find_magics.dir/src/bitboard.c.o"
	/usr/bin/clang $(C_DEFINES) $(C_INCLUDES) $(C_FLAGS) -o CMakeFiles/chess_find_magics.dir/src/bitboard.c.o   -c /home/edward/Documents/chess/src/bitboard.c

CMakeFiles/chess_find_magics.dir/src/bitboard.c.i: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green "Preprocessing C source to CMakeFiles/chess_find_magics.dir/src/bitboard.c.i"
	/usr/bin/clang $(C_DEFINES) $(C_INCLUDES) $(C_FLAGS) -E /home/edward/Documents/chess/src/bitboard.c > CMakeFiles/chess_find_magics.dir/src/bitboard.c.i

CMakeFiles/chess_find_magics.dir/src/bitboard.c.s: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green "Compiling C source to assembly CMakeFiles/chess_find_magics.dir/src/bitboard.c.s"
	/usr/bin/clang $(C_DEFINES) $(C_INCLUDES) $(C_FLAGS) -S /home/edward/Documents/chess/src/bitboard.c -o CMakeFiles/chess_find_magics.dir/src/bitboard.c.s

CMakeFiles/chess_find_magics.dir/src/move_gen.c.o: CMakeFiles/chess_find_magics.dir/flags.make
CMakeFiles/chess_find_magics.dir/src/move_gen.c.o: ../src/move_gen.c
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green --progress-dir=/home/edward/Documents/chess/cmake-build-debug/CMakeFiles --progress-num=$(CMAKE_PROGRESS_2) "Building C object CMakeFiles/chess_find_magics.dir/src/move_gen.c.o"
	/usr/bin/clang $(C_DEFINES) $(C_INCLUDES) $(C_FLAGS) -o CMakeFiles/chess_find_magics.dir/src/move_gen.c.o   -c /home/edward/Documents/chess/src/move_gen.c

CMakeFiles/chess_find_magics.dir/src/move_gen.c.i: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green "Preprocessing C source to CMakeFiles/chess_find_magics.dir/src/move_gen.c.i"
	/usr/bin/clang $(C_DEFINES) $(C_INCLUDES) $(C_FLAGS) -E /home/edward/Documents/chess/src/move_gen.c > CMakeFiles/chess_find_magics.dir/src/move_gen.c.i

CMakeFiles/chess_find_magics.dir/src/move_gen.c.s: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green "Compiling C source to assembly CMakeFiles/chess_find_magics.dir/src/move_gen.c.s"
	/usr/bin/clang $(C_DEFINES) $(C_INCLUDES) $(C_FLAGS) -S /home/edward/Documents/chess/src/move_gen.c -o CMakeFiles/chess_find_magics.dir/src/move_gen.c.s

CMakeFiles/chess_find_magics.dir/src/found_magics.c.o: CMakeFiles/chess_find_magics.dir/flags.make
CMakeFiles/chess_find_magics.dir/src/found_magics.c.o: ../src/found_magics.c
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green --progress-dir=/home/edward/Documents/chess/cmake-build-debug/CMakeFiles --progress-num=$(CMAKE_PROGRESS_3) "Building C object CMakeFiles/chess_find_magics.dir/src/found_magics.c.o"
	/usr/bin/clang $(C_DEFINES) $(C_INCLUDES) $(C_FLAGS) -o CMakeFiles/chess_find_magics.dir/src/found_magics.c.o   -c /home/edward/Documents/chess/src/found_magics.c

CMakeFiles/chess_find_magics.dir/src/found_magics.c.i: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green "Preprocessing C source to CMakeFiles/chess_find_magics.dir/src/found_magics.c.i"
	/usr/bin/clang $(C_DEFINES) $(C_INCLUDES) $(C_FLAGS) -E /home/edward/Documents/chess/src/found_magics.c > CMakeFiles/chess_find_magics.dir/src/found_magics.c.i

CMakeFiles/chess_find_magics.dir/src/found_magics.c.s: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green "Compiling C source to assembly CMakeFiles/chess_find_magics.dir/src/found_magics.c.s"
	/usr/bin/clang $(C_DEFINES) $(C_INCLUDES) $(C_FLAGS) -S /home/edward/Documents/chess/src/found_magics.c -o CMakeFiles/chess_find_magics.dir/src/found_magics.c.s

CMakeFiles/chess_find_magics.dir/src/find_magics.c.o: CMakeFiles/chess_find_magics.dir/flags.make
CMakeFiles/chess_find_magics.dir/src/find_magics.c.o: ../src/find_magics.c
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green --progress-dir=/home/edward/Documents/chess/cmake-build-debug/CMakeFiles --progress-num=$(CMAKE_PROGRESS_4) "Building C object CMakeFiles/chess_find_magics.dir/src/find_magics.c.o"
	/usr/bin/clang $(C_DEFINES) $(C_INCLUDES) $(C_FLAGS) -o CMakeFiles/chess_find_magics.dir/src/find_magics.c.o   -c /home/edward/Documents/chess/src/find_magics.c

CMakeFiles/chess_find_magics.dir/src/find_magics.c.i: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green "Preprocessing C source to CMakeFiles/chess_find_magics.dir/src/find_magics.c.i"
	/usr/bin/clang $(C_DEFINES) $(C_INCLUDES) $(C_FLAGS) -E /home/edward/Documents/chess/src/find_magics.c > CMakeFiles/chess_find_magics.dir/src/find_magics.c.i

CMakeFiles/chess_find_magics.dir/src/find_magics.c.s: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green "Compiling C source to assembly CMakeFiles/chess_find_magics.dir/src/find_magics.c.s"
	/usr/bin/clang $(C_DEFINES) $(C_INCLUDES) $(C_FLAGS) -S /home/edward/Documents/chess/src/find_magics.c -o CMakeFiles/chess_find_magics.dir/src/find_magics.c.s

# Object files for target chess_find_magics
chess_find_magics_OBJECTS = \
"CMakeFiles/chess_find_magics.dir/src/bitboard.c.o" \
"CMakeFiles/chess_find_magics.dir/src/move_gen.c.o" \
"CMakeFiles/chess_find_magics.dir/src/found_magics.c.o" \
"CMakeFiles/chess_find_magics.dir/src/find_magics.c.o"

# External object files for target chess_find_magics
chess_find_magics_EXTERNAL_OBJECTS =

chess_find_magics: CMakeFiles/chess_find_magics.dir/src/bitboard.c.o
chess_find_magics: CMakeFiles/chess_find_magics.dir/src/move_gen.c.o
chess_find_magics: CMakeFiles/chess_find_magics.dir/src/found_magics.c.o
chess_find_magics: CMakeFiles/chess_find_magics.dir/src/find_magics.c.o
chess_find_magics: CMakeFiles/chess_find_magics.dir/build.make
chess_find_magics: CMakeFiles/chess_find_magics.dir/link.txt
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green --bold --progress-dir=/home/edward/Documents/chess/cmake-build-debug/CMakeFiles --progress-num=$(CMAKE_PROGRESS_5) "Linking C executable chess_find_magics"
	$(CMAKE_COMMAND) -E cmake_link_script CMakeFiles/chess_find_magics.dir/link.txt --verbose=$(VERBOSE)

# Rule to build all files generated by this target.
CMakeFiles/chess_find_magics.dir/build: chess_find_magics

.PHONY : CMakeFiles/chess_find_magics.dir/build

CMakeFiles/chess_find_magics.dir/clean:
	$(CMAKE_COMMAND) -P CMakeFiles/chess_find_magics.dir/cmake_clean.cmake
.PHONY : CMakeFiles/chess_find_magics.dir/clean

CMakeFiles/chess_find_magics.dir/depend:
	cd /home/edward/Documents/chess/cmake-build-debug && $(CMAKE_COMMAND) -E cmake_depends "Unix Makefiles" /home/edward/Documents/chess /home/edward/Documents/chess /home/edward/Documents/chess/cmake-build-debug /home/edward/Documents/chess/cmake-build-debug /home/edward/Documents/chess/cmake-build-debug/CMakeFiles/chess_find_magics.dir/DependInfo.cmake --color=$(COLOR)
.PHONY : CMakeFiles/chess_find_magics.dir/depend

