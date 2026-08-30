cmake_minimum_required(VERSION 3.30)
project(BController VERSION 1.0.0 LANGUAGES CXX)

# Set the C standard to C11 or C17
set(CMAKE_CXX_STANDARD 23)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)
# set(CMAKE_CXX_IMPLICIT_INCLUDE_DIRECTORIES "")

find_package(PkgConfig REQUIRED)
find_package(Boost REQUIRED COMPONENTS headers url json)
find_package(OpenSSL REQUIRED)
find_package(Curses REQUIRED)

add_executable(b-controller src/main.cpp src/controller.cpp)
target_link_libraries(b-controller PRIVATE Boost::headers Boost::url Boost::json 
    OpenSSL::SSL OpenSSL::Crypto ${CURSES_LIBRARIES})

