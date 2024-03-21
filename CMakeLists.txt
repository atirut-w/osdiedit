add_executable("osdiedit"
    "src/main.cpp"
)
target_include_directories("osdiedit" PRIVATE "include")
target_link_libraries("osdiedit" PRIVATE "argparse" "tabulate" "osdi")

add_subdirectory("3rd/argparse")
add_subdirectory("3rd/tabulate")
add_subdirectory("osdi")
