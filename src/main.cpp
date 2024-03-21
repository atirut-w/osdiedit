#include <argparse/argparse.hpp>
#include <memory>
#include <osdi/osdi.hpp>
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
using namespace OSDI;

int sector_size;
int disk_size;
fstream image;
vector<Partition> partitions;
vector<string> changelog;

void list_partitions(vector<string>)
{
    cout << "Partition list for " << partitions[0].label << " (" << disk_size << " sectors)" << endl;
    Table partitions_table;
    partitions_table.format()
        .column_separator(""); // TODO: PR a fix for this

    partitions_table.add_row({"ID", "Start", "Size", "Type", "Label"});
    for (int i = 1; i < partitions.size(); i++)
    {
        partitions_table.add_row({
            to_string(i),
            to_string(partitions[i].start),
            to_string(partitions[i].size),
            partitions[i].type,
            string(partitions[i].label) + (partitions[i].flags & Partition::ACTIVE ? "*" : " ")
        });
    }
    cout << partitions_table << endl;

    if (changelog.size() > 0)
        cout << "Warning: uncommitted changes" << endl;
}

int getchar_fixed()
{
    int c = getchar();
    if (c != '\n')
        cin.ignore(numeric_limits<streamsize>::max(), '\n');
    return c;
}

void active_partition(vector<string> args)
{
    if (args.size() < 2)
    {
        for (int i = 0; i < partitions.size(); i++)
        {
            if (partitions[i].flags & Partition::ACTIVE)
            {
                cout << "Current active partition: " << partitions[i].label << " (partition " << i << ")" << endl;
            }
        }
    }
    else
    {
        int part_id = stoi(args[1]);
        if (part_id < 1 || part_id >= partitions.size())
        {
            cerr << "Invalid partition ID" << endl;
            return;
        }
        
        for (int i = 0; i < partitions.size(); i++)
        {
            partitions[i].flags &= ~Partition::ACTIVE;
        }
        partitions[part_id].flags |= Partition::ACTIVE;
        changelog.push_back("Set partition " + to_string(part_id) + " as active");
    }
}

void commit(vector<string>)
{
    if (changelog.size() == 0)
    {
        cout << "Nothing to commit" << endl;
        return;
    }
    
    cout << "Changes so far:" << endl;
    for (auto &change : changelog)
    {
        cout << " - " << change << endl;
    }
    cout << "Commit changes? [y/N]: ";
    if (tolower(getchar_fixed()) != 'y')
        return;
    
    image.seekp(0);
    for (auto &part : partitions)
    {
        part.write(image);
    }
    for (int i = 0; i < sector_size - (sizeof(Partition) * partitions.size()); i++)
    {
        image.write("\0", 1);
    }
    image.flush();

    cout << "Committed " << changelog.size() << " changes" << endl;
    changelog.clear();
}

map<string, function<void(vector<string>)>> commands = {
    {"exit", [](vector<string>) { exit(0); }},
    {"list", list_partitions},
    {"active", active_partition},
    {"commit", commit},
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
    image.seekg(0, ios::end);
    disk_size = image.tellg() / sector_size;
    image.seekg(0);

    cout << "Reading partition table..." << endl;
    for (int i = 0; i < sector_size / sizeof(Partition); ++i)
    {
        Partition partition = Partition(image);
        
        if (partition.type != "")
            partitions.push_back(partition);
    }

    if (partitions[0].size != 0 || string(partitions[0].type) != "OSDI\xaa\xaa\x55\x55")
    {
        cout << "Not an OSDI image. Create new partition table? [y/N] ";

        if (tolower(getchar_fixed()) != 'y')
            return 1;

        image.seekp(0);
        image.write("\0", sector_size);
        image.seekp(0);

        Partition signature;
        signature.start = 1; // Only version 1 is supported
        signature.size = 0;
        signature.type = "OSDI\xaa\xaa\x55\x55";
        signature.flags = 0;

        signature.label = "osdi-";
        srand(time(nullptr));
        for (int i = 0; i < (13 - signature.label.length()); ++i)
            signature.label += '0' + (rand() % 10);

        signature.write(image);
        partitions.clear();
        partitions.push_back(signature);
    }
    else if (partitions[0].start != 1)
    {
        cout << "Unsupported partition table version" << endl;
        return 1;
    }
    else
    {
        cout << "Read in " << partitions.size() - 1 << " partitions" << endl;
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

        try
        {
            commands[words[0]](words);
        }
        catch (const exception &err)
        {
            cerr << err.what() << endl;
        }
    }
    
    return 0;
}
