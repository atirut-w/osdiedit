add_executable("osdiedit"
    "src/main.cpp"
)
target_include_directories("osdiedit" PRIVATE "include")
target_link_libraries("osdiedit" PRIVATE "argparse" "tabulate")

add_subdirectory("3rd/argparse")
add_subdirectory("3rd/tabulate")
