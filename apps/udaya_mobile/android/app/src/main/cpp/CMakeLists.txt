cmake_minimum_required(VERSION 3.22.1)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

project("udaya_ffi_mobile")

find_library(log-lib log)

add_library(udaya_ffi_mobile SHARED IMPORTED)
set_target_properties(udaya_ffi_mobile PROPERTIES IMPORTED_LOCATION ${CMAKE_SOURCE_DIR}/../jniLibs/${ANDROID_ABI}/libudaya_ffi_mobile.so)

add_library(udaya_ffi_bridge SHARED udaya_ffi_bridge.cpp)
target_link_libraries(udaya_ffi_bridge udaya_ffi_mobile ${log-lib})
