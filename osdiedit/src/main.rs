use std::{collections::HashMap, fs::File, io::Write};

use clap::Parser;
use osdi::Disk;
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
    execute: Box<dyn FnMut(&Disk, Vec<String>) -> Result<(), String>>,
}

impl Command {
    pub fn new(
        description: String,
        execute: Box<dyn FnMut(&Disk, Vec<String>) -> Result<(), String>>,
    ) -> Self {
        Command {
            description,
            execute,
        }
    }
}

fn info(disk: &Disk, _args: Vec<String>) -> Result<(), String> {
    println!("Sector Size: {} bytes", disk.sector_size);
    println!("Total Size in Sectors: {}", disk.size);
    println!("Number of Partitions: {}", disk.partitions.len());
    Ok(())
}

fn list(disk: &Disk, _args: Vec<String>) -> Result<(), String> {
    let partitions = &disk.partitions;
    if partitions.is_empty() {
        println!("No partitions found.");
        return Ok(());
    }

    println!(
        "{:<5} {:<8} {:<12} {:<8} {:<8} {:<8} {:<8}",
        "Idx", "Type", "Name", "Start", "End", "Size", "Flags"
    );
    for (idx, partition) in partitions.iter().enumerate() {
        let start = partition.start;
        let size = partition.data.len() / disk.sector_size;
        let end = start + size as u32;
        println!(
            "{:<5} {:<8} {:<12} {:<8} {:<8} {:<8} {:<8x}",
            idx,
            String::from_utf8_lossy(&partition.type_id).trim_end_matches('\0'),
            String::from_utf8_lossy(&partition.name).trim_end_matches('\0'),
            start,
            end,
            size,
            partition.flags,
        );
    }

    Ok(())
}

fn main() {
    let args = Args::parse();
    let file = match File::open(&args.disk) {
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

    let mut commands: HashMap<String, Command> = HashMap::new();
    commands.insert(
        "info".to_string(),
        Command::new("Show disk information".to_string(), Box::new(info)),
    );
    commands.insert(
        "list".to_string(),
        Command::new("List all partitions".to_string(), Box::new(list)),
    );

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
                println!("{:<10} {:<8}", "exit", "Exit the program");
                for (name, command) in &commands {
                    println!("{:<10} {:<8}", name, command.description);
                }
            }
            "exit" => {
                println!("Exiting...");
                break;
            }
            _ => {
                if let Some(command) = commands.get_mut(command_name) {
                    match (command.execute)(&disk, args[1..].to_vec()) {
                        Ok(_) => {}
                        Err(e) => eprintln!("{}", e),
                    }
                } else {
                    eprintln!("Unknown command: {}", command_name);
                }
            }
        }
    }
}
