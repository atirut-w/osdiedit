#include <argparse/argparse.hpp>
#include <memory>

using namespace std;
using namespace argparse;

shared_ptr<const ArgumentParser> parse_args(int argc, char *argv[])
{
    auto parser = make_shared<ArgumentParser>("osdiedit");

    parser->add_argument("image")
        .help("Disk image to edit");
    
    parser->add_argument("-s", "--sector-size")
        .help("Sector size")
        .default_value(512)
        .scan<'i', int>();
    
    try
    {
        parser->parse_args(argc, argv);
    }
    catch (const exception &err)
    {
        cerr << err.what() << endl;
        cerr << parser;
        return nullptr;
    }
    return parser;
}

int main(int argc, char *argv[])
{
    auto args = parse_args(argc, argv);
    if (!args)
        return 1;
    
    return 0;
}
