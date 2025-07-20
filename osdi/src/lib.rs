use std::{
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
};

use byteorder::{LittleEndian, ReadBytesExt};

pub struct Disk {
    pub sector_size: usize,
    pub partitions: Vec<Partition>,
}

impl Disk {
    pub fn from_file(
        mut reader: BufReader<&File>,
        sector_size: usize,
    ) -> Result<Self, std::io::Error> {
        let mut partitions = vec![];

        for i in 0..(sector_size / 32) {
            let start = reader.read_u32::<LittleEndian>()?;
            let size = reader.read_u32::<LittleEndian>()?;
            let mut type_id = [0; 8];
            reader.read_exact(&mut type_id)?;
            let mut flags = reader.read_u8()? as u32;
            flags |= (reader.read_u8()? as u32) << 8;
            flags |= (reader.read_u8()? as u32) << 16;
            let mut name = [0; 13];
            reader.read_exact(&mut name)?;

            if i == 0 {
                if type_id != *b"OSDI\xaa\xaa\x55\x55" || size != 0 {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid OSDI disk signature",
                    ));
                };
                continue;
            }

            if type_id == [0; 8] {
                continue;
            }

            partitions.push(Partition {
                start: start - 1, // Convert to zero-based index
                type_id,
                flags,
                name,
                data: vec![0; (size as usize) * sector_size],
            });
        }

        for partition in &mut partitions {
            reader.seek(SeekFrom::Start(partition.start as u64 * sector_size as u64))?;
            reader.read_exact(&mut partition.data)?;
        }

        Ok(Disk {
            sector_size,
            partitions,
        })
    }
}

pub struct Partition {
    pub start: u32,
    pub type_id: [u8; 8],
    pub flags: u32,
    pub name: [u8; 13],
    pub data: Vec<u8>,
}
