use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, BufWriter, Read, Seek, Write},
};

use clap::Parser;
use dialoguer::Confirm;
use osdi::{Disk, Partition};
use shell_words::split;

#[derive(Parser)]
struct Args {
    /// The disk to edit
    disk: String,

    /// The sector size of the disk
    #[clap(short, long, default_value = "512")]
    sector_size: usize,
}

struct Command {
    description: String,
    execute: Box<dyn FnMut(&mut Context, &Vec<String>) -> Result<bool, String>>,
}

impl Command {
    pub fn new(
        description: String,
        execute: Box<dyn FnMut(&mut Context, &Vec<String>) -> Result<bool, String>>,
    ) -> Self {
        Command {
            description,
            execute,
        }
    }
}

struct Context {
    file: File,
    disk: Disk,
}

fn validate_partid(disk: &Disk, partid: usize) -> Result<&Partition, String> {
    if partid >= disk.partitions.len() {
        return Err(format!("Partition index {} does not exist", partid));
    }
    Ok(&disk.partitions[partid])
}

fn info(context: &mut Context, _args: &Vec<String>) -> Result<bool, String> {
    let disk = &context.disk;
    println!(
        "Information for disk '{}':",
        String::from_utf8_lossy(&disk.label).trim_end_matches('\0')
    );
    println!("Sector Size: {} bytes", disk.sector_size);
    println!("Total Size in Sectors: {}", disk.size);
    println!("Number of Partitions: {}", disk.partitions.len());
    Ok(false)
}

fn list(context: &mut Context, _args: &Vec<String>) -> Result<bool, String> {
    let partitions = &context.disk.partitions;
    if partitions.is_empty() {
        println!("No partitions found.");
        return Ok(false);
    }

    println!(
        "{:<5} {:<8} {:<12} {:<8} {:<8} {:<8} {:<8}",
        "Idx", "Type", "Name", "Start", "Size", "End", "Flags"
    );
    for (idx, partition) in partitions.iter().enumerate() {
        let start = partition.start;
        let end = start + partition.size - 1;
        println!(
            "{:<5} {:<8} {:<12} {:<8} {:<8} {:<8} {:<8}",
            idx,
            partition.get_type_id(),
            partition.get_name(),
            start,
            partition.size,
            end,
            partition.query_flags().join(", "),
        );
    }

    Ok(false)
}

#[derive(Parser)]
struct DumpArgs {
    /// The index of the partition to dump
    index: usize,

    /// The file to dump the partition to
    file: String,
}

fn dump(context: &mut Context, args: &Vec<String>) -> Result<bool, String> {
    let dump_args = DumpArgs::try_parse_from(args).map_err(|e| e.to_string())?;

    let partition = validate_partid(&context.disk, dump_args.index)?;

    let file = match File::create(&dump_args.file) {
        Ok(file) => file,
        Err(e) => {
            return Err(format!(
                "Error creating dump file {}: {}",
                dump_args.file, e
            ));
        }
    };

    println!(
        "Dumping partition '{}' (index {}) to file '{}'...",
        partition.get_name(),
        dump_args.index,
        dump_args.file
    );

    let mut reader = BufReader::new(&context.file);
    let mut writer = BufWriter::new(file);

    reader.seek(std::io::SeekFrom::Start(
        partition.start as u64 * context.disk.sector_size as u64,
    )).map_err(|e| format!("Error seeking to partition start: {}", e))?;
    for sector in partition.start..(partition.start + partition.size) {
        let mut sector_data = vec![0u8; context.disk.sector_size];
        reader
            .read_exact(&mut sector_data)
            .map_err(|e| format!("Error reading sector {}: {}", sector, e))?;
        writer
            .write_all(&sector_data)
            .map_err(|e| format!("Error writing to dump file: {}", e))?;
    }

    writer
        .flush()
        .map_err(|e| format!("Error flushing dump file: {}", e))?;
    println!(
        "Partition '{}' dumped successfully to '{}'.",
        partition.get_name(),
        dump_args.file
    );
    Ok(false)
}

fn main() {
    let args = Args::parse();
    let file = match File::options().read(true).write(true).open(&args.disk) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error opening disk {}: {}", args.disk, e);
            return;
        }
    };

    let disk = match Disk::from_file(std::io::BufReader::new(&file), args.sector_size) {
        Ok(disk) => disk,
        Err(e) => {
            eprintln!("Error reading disk {}: {}", args.disk, e);
            return;
        }
    };

    let mut context = Context { file, disk };

    let mut commands: HashMap<String, Command> = HashMap::new();
    commands.insert(
        "info".to_string(),
        Command::new("Show disk information".to_string(), Box::new(info)),
    );
    commands.insert(
        "list".to_string(),
        Command::new("List all partitions".to_string(), Box::new(list)),
    );
    commands.insert(
        "dump".to_string(),
        Command::new("Dump a partition to a file".to_string(), Box::new(dump)),
    );

    let mut changelog: Vec<String> = Vec::new();

    println!("Type 'help' for a list of commands.");
    loop {
        let mut input = String::new();
        print!("> ");
        std::io::stdout().flush().expect("Failed to flush stdout");
        if std::io::stdin().read_line(&mut input).is_err() {
            eprintln!("Error reading input");
            break;
        }

        let args = match split(input.trim()) {
            Ok(args) => args,
            Err(e) => {
                eprintln!("Error parsing command: {}", e);
                continue;
            }
        };

        if args.is_empty() {
            continue;
        }

        let command_name = &args[0];
        match command_name.as_str() {
            "help" => {
                println!("{:<10} {:<8}", "Command", "Description");
                println!("{:<10} {:<8}", "help", "Show this help");
                println!("{:<10} {:<8}", "commit", "Commit changes to disk");
                println!("{:<10} {:<8}", "exit", "Exit the program");
                for (name, command) in &commands {
                    println!("{:<10} {:<8}", name, command.description);
                }
            }
            "commit" => {
                if changelog.is_empty() {
                    println!("No changes to commit.");
                    continue;
                }

                println!("You have made the following changes:");
                for change in &changelog {
                    println!("- {}", change);
                }
                if Confirm::new()
                    .with_prompt("Do you want to commit these changes?")
                    .default(true)
                    .interact()
                    .unwrap_or(false)
                {
                    println!("Committing changes...");
                    let writer = std::io::BufWriter::new(&context.file);
                    if let Err(e) = context.disk.to_file(writer) {
                        eprintln!("Error writing to disk: {}", e);
                    } else {
                        println!("Changes committed successfully.");
                        changelog.clear();
                    }
                }
            }
            "exit" => {
                println!("Exiting...");
                break;
            }
            _ => {
                if let Some(command) = commands.get_mut(command_name) {
                    match (command.execute)(&mut context, &args) {
                        Ok(changed) => {
                            if changed {
                                changelog.push(args.join(" "));
                            }
                        }
                        Err(e) => eprintln!("{}", e),
                    }
                } else {
                    eprintln!("Unknown command: {}", command_name);
                }
            }
        }
    }
}
