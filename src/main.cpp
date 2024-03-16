#include <argparse/argparse.hpp>
#include <memory>
#include <osdi.hpp>
#include <fstream>
#include <vector>
#include <iostream>
#include <cstring>
#include <map>
#include <functional>
#include <string>
#include <tabulate/table.hpp>

using namespace std;
using namespace argparse;
using namespace tabulate;

int sector_size;
fstream image;
vector<OSDIPartition> partitions;

void list_partitions(int argc, char *argv[])
{
    cout << "Listing partitinos for " << partitions[0].name << endl;
    Table partitions_table;
    partitions_table.format()
        .column_separator(""); // TODO: PR a fix for this

    partitions_table.add_row({"ID", "Start sector", "Size", "Type", "Label"});

    for (int i = 1; i < partitions.size(); i++)
    {
        partitions_table.add_row({
            to_string(i),
            to_string(partitions[i].start_sector),
            to_string(partitions[i].size * sector_size),
            partitions[i].type,
            string(partitions[i].flags[1] & 0x02 ? "*" : "") + partitions[i].name
        });
    }

    cout << partitions_table << endl;
}

map<string, function<void(int argc, char *argv[])>> commands = {
    {"exit", [](int argc, char *argv[]) { exit(0); }},
    {"list", list_partitions}
};

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
    
    image.open(args->get("image"), ios::in | ios::out | ios::binary);
    if (!image)
    {
        cerr << "Failed to open disk image" << endl;
        return 1;
    }

    sector_size = args->get<int>("-s");
    cout << "Reading partition table..." << endl;
    for (int i = 0; i < sector_size / sizeof(OSDIPartition); ++i)
    {
        OSDIPartition partition;
        image.read(reinterpret_cast<char *>(&partition), sizeof(OSDIPartition));
        
        if (*(uint64_t *)&partition.type != 0)
            partitions.push_back(partition);
    }

    if (partitions[0].size != 0 || string(partitions[0].type) != "OSDI\xaa\xaa\x55\x55")
    {
        cout << "Not an OSDI image. Create new partition table? [y/N] ";

        char c;
        cin >> c;
        cin.ignore(numeric_limits<streamsize>::max(), '\n');
        if (c != 'Y' && c != 'y')
            return 0;

        image.seekp(0);
        image.write("\0", sector_size);
        image.seekp(0);

        OSDIPartition signature;
        signature.start_sector = 1; // Only version 1 is supported
        signature.size = 0;
        memcpy(signature.type, "OSDI\xaa\xaa\x55\x55", 8);
        memset(signature.flags, 0, 3);

        memcpy(signature.name, "osdi-", 5);
        srand(time(nullptr));
        for (int i = 0; i < 8; ++i)
            signature.name[5 + i] = '0' + (rand() % 10);
        
        image.write(reinterpret_cast<const char *>(&signature), sizeof(OSDIPartition));
        partitions.clear();
        partitions.push_back(signature);
    }
    else if (partitions[0].start_sector != 1)
    {
        cout << "Unsupported partition table version" << endl;
        return 1;
    }
    else
    {
        cout << "Read in " << partitions.size() << " partitions" << endl;
    }

    cout << "Type `help` for a list of available commands" << endl;
    while (1)
    {
        string cmd;
        cout << "> ";
        getline(cin, cmd);

        vector<string> words;
        string word;
        stringstream ss(cmd);
        while (getline(ss, word, ' '))
            words.push_back(word);
        
        if (words.size() == 0)
        {
            continue;
        }
        else if (words[0] == "help")
        {
            for (auto pair : commands)
            {
                cout << pair.first << endl;
            }
            continue;
        }
        else if (commands.count(words[0]) == 0)
        {
            cerr << "Unknown command: " << words[0] << endl;
            continue;
        }

        vector<char*> argv;
        for (auto w : words)
            argv.push_back(const_cast<char *>(w.c_str()));

        try
        {
            commands[cmd](argv.size(), argv.data());
        }
        catch (const exception &err)
        {
            cerr << err.what() << endl;
        }
    }
    
    return 0;
}
