use std::fs::File;

use clap::Parser;
use osdi::Disk;

#[derive(Parser)]
struct Args {
    /// The disk to edit
    disk: String,

    /// The sector size of the disk
    #[clap(short, long, default_value = "512")]
    sector_size: usize,
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

    for partition in &disk.partitions {
        println!("Partition: {} (Type: {}, Flags: {:x})", 
                 String::from_utf8_lossy(&partition.name), 
                 String::from_utf8_lossy(&partition.type_id), 
                 partition.flags);
        println!("Start: {}, Size: {} bytes", 
                 partition.start, 
                 partition.data.len());
    }
}
