use std::fs::File;

use clap::Parser;

#[derive(Parser)]
struct Args {
    /// The disk to edit
    disk: String,
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
}
