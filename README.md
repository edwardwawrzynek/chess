# Chess Programming Competition
A competition to produce a program to play chess.

Standard rules are used, except that:
- Threefold and fivefold repetition aren't present
- The fifty and seventy-five move rules aren't present

This repository contains the following components:
- A server to coordinate play between programs, along with a frontend for humans to interact with the server. The frontend is deployed at: [https://codekata-chess.herokuapp.com](https://codekata-chess.herokuapp.com).
  - If you want to interact directly with the server, open a websocket connection to ws://codekata-chess.herokuapp.com:80.
- A C chess utility library with a fast legal move generator and other tools (`lib`)
  - the `perft_tests` testsuite (test driver: `utils/perft_test.c`)
  - a sample minimax implemented with the c library (`utils/sample.c`)
- C++, Go, and Rust bindings to the utility library (see below)

## Server and Frontend
The server is deployed at [https://codekata-chess.herokuapp.com](https://codekata-chess.herokuapp.com). It is recommend to test your programs against this deployment. If you want to run the server locally, use the included `Dockerfile`.

## C Library
The C library is located in `lib`. To build and install:
```
mkdir build
cd build
cmake ..
cmake --build . --target chess-util
sudo cmake --install . --component chess-util
```
The c library installs a `chess-util.h` header and can be linked as `chess-util`.

## C++ Library
The C++ library is located in `cpp_binding`/. Documentation for the C++ library can be found at: [https://edward.wawrzynek.com/chess](https://edward.wawrzynek.com/chess).

The C++ library requires [boost](https://boost.org) to be installed. To build and install the C++ library:
```
mkdir build
cd build
cmake ..
cmake --build . --target chess-util chess-cpp
sudo cmake --install . --component chess-util
sudo cmake --install . --component chess-cpp
```
The c++ library installs a `chess.hpp` header and can be linked as `chess-cpp`.

### Example C++ Program
The `cpp_example/` folder contains an example minimax implemented using the C++ library. To build and run it:
```
mkdir build
cd build
cmake ..
cmake --build . --target cpp_example
./cpp_example/cpp_example codekata-chess.herokuapp.com 80 API_KEY "Sample C++ Program"
```
The example program is a good place to start. If your program expands into multiple files beyond `example.cpp`, you need to add those new files to `cpp_example/CMakeLists.txt` (see the comment at the top of that file on how to do that).

## Go Library
The Go library is located in `go_chess/`. Documentation can be found with `go doc`.

Installation:
- [Install the C library](#c-library)
- Install [`cgo`](https://golang.org/cmd/cgo/) for your platform (your go installation may come with cgo by default)
- `go get github.com/edwardwawrzynek/chess/go_chess`

### Example Go Program
The `go_example/` folder contains an example minimax implemented using the C++ library. To run it:
```
cd go_example
go run example.go codekata-chess.herokuapp.com 80 API_KEY "Sample Go Program"
```

## Rust Library
The rust bindings are not yet complete.

## Library Debug / Release Builds
By default, cmake builds the library in debug mode. The library is full of assertions and invariant checks, which ensure that the library is functioning properly and that your program isn't passing it bad data. Additionally, debug builds can be easily debugged with a c/c++ debugger.

Release builds disable all of these assertions and will run **significantly** faster. However, the library may silently fail or behave erroneously if input invariants aren't met. You should always develop your program against debug builds of the libraries, then link them against release builds.

To have cmake create release builds, add the `--config Release` flag to all cmake commands.

## Resources
- [Chess Programming Wiki](https://www.chessprogramming.org/Main_Page)
- [Computer Chess Programming Theory](http://frayn.net/beowulf/theory.html)

## Questions / Issues?
- If you have any issues, reach out to me (Edward Wawrzynek) on slack or at <edward@wawrzynek.com>.
- If you think you've found a bug, please file an issue using github or contact me.
- Improvements are welcome (create a pull request using github).
