use std::{collections::HashMap, fs::File, io::Write};

use clap::Parser;
use dialoguer::Confirm;
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
    execute: Box<dyn FnMut(&Disk, Vec<String>) -> Result<bool, String>>,
}

impl Command {
    pub fn new(
        description: String,
        execute: Box<dyn FnMut(&Disk, Vec<String>) -> Result<bool, String>>,
    ) -> Self {
        Command {
            description,
            execute,
        }
    }
}

fn info(disk: &Disk, _args: Vec<String>) -> Result<bool, String> {
    println!("Information for disk '{}':", String::from_utf8_lossy(&disk.label).trim_end_matches('\0'));
    println!("Sector Size: {} bytes", disk.sector_size);
    println!("Total Size in Sectors: {}", disk.size);
    println!("Number of Partitions: {}", disk.partitions.len());
    Ok(false)
}

fn list(disk: &Disk, _args: Vec<String>) -> Result<bool, String> {
    let partitions = &disk.partitions;
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

    let mut commands: HashMap<String, Command> = HashMap::new();
    commands.insert(
        "info".to_string(),
        Command::new("Show disk information".to_string(), Box::new(info)),
    );
    commands.insert(
        "list".to_string(),
        Command::new("List all partitions".to_string(), Box::new(list)),
    );

    let mut changelog = vec![];

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
                    let writer = std::io::BufWriter::new(&file);
                    if let Err(e) = disk.to_file(writer) {
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
                    match (command.execute)(&disk, args[1..].to_vec()) {
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
