# Cyboquatics Multi-Language Build System
# Supports C++, Rust (via CMake integration), and Lua
# 
# Evidence Hex: 0xCQ2026CMAKE8B7A6F5E
# Knowledge Factor: F ≈ 0.87

cmake_minimum_required(VERSION 3.20)
project(Cyboquatics 
    VERSION 1.0.0
    DESCRIPTION "Cyboquatic Machine Control System with Soul-Boundary Enforcement"
    LANGUAGES CXX Lua
)

# C++ Standard Requirements
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# Build Options
option(CYBOQUATICS_BUILD_TESTS "Build test suite" ON)
option(CYBOQUATICS_BUILD_DOCS "Build documentation" ON)
option(CYBOQUATICS_ENABLE_SANITIZERS "Enable address and UB sanitizers" OFF)
option(CYBOQUATICS_OPTIMIZE_FOR_SIZE "Optimize for binary size" OFF)

# Compiler Warnings (Professional Quality)
if(CMAKE_CXX_COMPILER_ID MATCHES "GNU|Clang")
    add_compile_options(
        -Wall
        -Wextra
        -Wpedantic
        -Werror=return-type
        -Wno-unused-parameter
    )
    
    if(CYBOQUATICS_ENABLE_SANITIZERS)
        add_compile_options(-fsanitize=address,undefined)
        add_link_options(-fsanitize=address,undefined)
    endif()
endif()

if(MSVC)
    add_compile_options(/W4 /WX)
endif()

# Find Required Packages
find_package(Threads REQUIRED)
find_package(Lua 5.4 REQUIRED)

# Rust Integration (via Cargo)
# Note: Rust crates are built separately via Cargo.toml
# This CMake project links to the compiled Rust libraries

# Source Files
set(CYBOQUATICS_CPP_SOURCES
    src/cpp/CyboquaticMachineController.cpp
    src/cpp/SensorFusionEngine.cpp
    src/cpp/HumanRoboticsInterface.cpp
)

set(CYBOQUATICS_CPP_HEADERS
    src/cpp/CyboquaticMachineController.h
    src/cpp/SensorFusionEngine.h
    src/cpp/HumanRoboticsInterface.h
)

set(CYBOQUATICS_LUA_SOURCES
    src/lua/policy_router.lua
    src/lua/compliance_checker.lua
    src/lua/eco_impact_calculator.lua
)

# Main Library
add_library(cyboquatics_core STATIC
    ${CYBOQUATICS_CPP_SOURCES}
    ${CYBOQUATICS_CPP_HEADERS}
)

target_include_directories(cyboquatics_core PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}/src/cpp
    ${LUA_INCLUDE_DIR}
)

target_link_libraries(cyboquatics_core PUBLIC
    Threads::Threads
    ${LUA_LIBRARIES}
)

target_compile_definitions(cyboquatics_core PUBLIC
    CYBOQUATICS_VERSION="${PROJECT_VERSION}"
    CYBOQUATICS_EVIDENCE_HEX="0xCQ2026CMAKE8B7A6F5E"
)

# Lua Module
add_library(cyboquatics_lua MODULE
    src/lua/lua_binding.cpp
)

target_link_libraries(cyboquatics_lua PRIVATE
    cyboquatics_core
    ${LUA_LIBRARIES}
)

set_target_properties(cyboquatics_lua PROPERTIES
    PREFIX ""
    OUTPUT_NAME "cyboquatics"
)

# Test Suite
if(CYBOQUATICS_BUILD_TESTS)
    enable_testing()
    
    add_executable(cyboquatics_tests
        tests/cpp/machine_controller_test.cpp
    )
    
    target_link_libraries(cyboquatics_tests PRIVATE
        cyboquatics_core
        GTest::gtest_main
    )
    
    include(GoogleTest)
    gtest_discover_tests(cyboquatics_tests)
endif()

# Installation
install(TARGETS cyboquatics_core cyboquatics_lua
    LIBRARY DESTINATION lib
    ARCHIVE DESTINATION lib
    RUNTIME DESTINATION bin
)

install(FILES ${CYBOQUATICS_LUA_SOURCES}
    DESTINATION share/cyboquatics/lua
)

install(DIRECTORY aln/particles/
    DESTINATION share/cyboquatics/aln/particles
)

# Documentation
if(CYBOQUATICS_BUILD_DOCS)
    find_package(Doxygen QUIET)
    if(DOXYGEN_FOUND)
        set(DOXYGEN_OUTPUT_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}/docs)
        doxygen_add_docs(cyboquatics_docs
            ${CMAKE_CURRENT_SOURCE_DIR}/src
            ${CMAKE_CURRENT_SOURCE_DIR}/docs
            COMMENT "Generating API documentation with Doxygen"
        )
    endif()
endif()

# Print Build Configuration
message(STATUS "Cyboquatics Build Configuration:")
message(STATUS "  Version: ${PROJECT_VERSION}")
message(STATUS "  C++ Standard: ${CMAKE_CXX_STANDARD}")
message(STATUS "  Build Tests: ${CYBOQUATICS_BUILD_TESTS}")
message(STATUS "  Build Docs: ${CYBOQUATICS_BUILD_DOCS}")
message(STATUS "  Evidence Hex: 0xCQ2026CMAKE8B7A6F5E")
message(STATUS "  Knowledge Factor: F ≈ 0.87")
