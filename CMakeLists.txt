cmake_minimum_required(VERSION 3.10)

project(tmux-status)

set(CMAKE_BUILD_TYPE Release)

file(GLOB SOURCES "src/*.cpp")

set(CMAKE_CXX_FLAGS_RELEASE "-O3")
set(CMAKE_CXX_STANDARD 20)

add_executable(tmux-status ${SOURCES})
