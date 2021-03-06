FROM ubuntu:20.04
MAINTAINER Edward Wawrzynek <edward@wawrzynek.com>

# Install deps
RUN apt-get update && DEBIAN_FRONTEND="noninteractive" apt-get install -y build-essential make cmake git zlib1g-dev libboost-all-dev nodejs npm

# Install uWebSockets
WORKDIR /usr/src/
RUN git clone --branch v18.23.0 --recursive https://github.com/uNetworking/uWebSockets.git
RUN make -C uWebSockets
RUN make -C uWebSockets install
RUN ln `pwd`/uWebSockets/uSockets/uSockets.a /usr/local/lib/libusockets.a
RUN ln `pwd`/uWebSockets/uSockets/src/libusockets.h /usr/local/include/libusockets.h
RUN ldconfig

# Build Project
WORKDIR /usr/src/chess/
COPY . .
RUN mkdir build
WORKDIR /usr/src/chess/build/
RUN cmake ..
RUN cmake --build . --target chess_server chess-util chess-cpp
RUN cmake --install .
RUN ldconfig

# Build frontend
WORKDIR /usr/src/chess/frontend
RUN npm install
RUN npm run build

EXPOSE 9001
CMD chess_server /usr/src/chess/frontend/build
