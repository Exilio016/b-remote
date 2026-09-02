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
find_package(raylib REQUIRED)
find_package(raylib_cpp QUIET)
if (NOT raylib_cpp_FOUND)
    if (NOT DEFINED RAYLIB_CPP_VERSION)
        set(RAYLIB_CPP_VERSION v6.0.0)
    endif()
    include(FetchContent)
    FetchContent_Declare(
        raylib_cpp
        GIT_REPOSITORY https://github.com/RobLoach/raylib-cpp.git
        GIT_TAG ${RAYLIB_CPP_VERSION}
    )
    FetchContent_MakeAvailable(raylib_cpp)
endif()

add_library(b-controller-core src/controller.cpp)
target_link_libraries(b-controller-core PRIVATE Boost::headers Boost::url Boost::json 
    OpenSSL::SSL OpenSSL::Crypto)

add_executable(b-controller-tui src/main-tui.cpp)
target_link_libraries(b-controller-tui PRIVATE b-controller-core ${CURSES_LIBRARIES})

add_executable(b-controller src/main.cpp)
target_link_libraries(b-controller PRIVATE b-controller-core raylib raylib_cpp)

install(TARGETS b-controller-tui DESTINATION bin)
install(TARGETS b-controller DESTINATION bin)
install(DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}/assets"
        DESTINATION bin)

set(DESKTOP_FILE "${CMAKE_CURRENT_BINARY_DIR}/${PROJECT_NAME}.desktop")
configure_file(
    "${CMAKE_CURRENT_SOURCE_DIR}/template.desktop.in"
    "${DESKTOP_FILE}"
    @ONLY
)
install(
    FILES "${DESKTOP_FILE}"
    DESTINATION "share/applications"
)
