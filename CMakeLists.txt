cmake_minimum_required(VERSION 3.20)
project(cyboquatics-phoenix C CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_POSITION_INDEPENDENT_CODE ON)

include_directories(${CMAKE_CURRENT_SOURCE_DIR}/include)

add_library(cybo_cpvm STATIC
    src/cybo_cpvm.cpp
)
target_include_directories(cybo_cpvm PUBLIC include)

add_library(cybo_ceim STATIC
    src/cybo_ceim.cpp
    src/cybo_qpudata_csv.cpp
)
target_include_directories(cybo_ceim PUBLIC include)

add_executable(phx_sim
    src/main_phx_sim.cpp
)
target_link_libraries(phx_sim PRIVATE cybo_cpvm cybo_ceim)
