use std::{
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
};

use byteorder::{LittleEndian, ReadBytesExt};

pub struct Disk {
    pub sector_size: usize,
    pub size: usize,
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

        let size = reader.seek(SeekFrom::End(0))? / sector_size as u64;

        Ok(Disk {
            sector_size,
            size: size as usize,
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

impl Partition {
    pub fn query_flags(&self) -> Vec<String> {
        let mut flags = vec![];
        
        if self.flags & PartitionFlags::OS as u32 != 0 {
            flags.push("OS".to_string());
        }
        if self.flags & PartitionFlags::Bootloader as u32 != 0 {
            flags.push("Bootloader".to_string());
        }
        if self.flags & PartitionFlags::PosixPermissions as u32 != 0 {
            flags.push("POSIX".to_string());
        }
        if self.flags & PartitionFlags::ReadOnly as u32 != 0 {
            flags.push("RO".to_string());
        }
        if self.flags & PartitionFlags::Hidden as u32 != 0 {
            flags.push("Hidden".to_string());
        }
        if self.flags & PartitionFlags::System as u32 != 0 {
            flags.push("System".to_string());
        }
        if self.flags & PartitionFlags::Zorya as u32 != 0 {
            flags.push("Zorya".to_string());
        }
        if self.flags & PartitionFlags::Managed as u32 != 0 {
            flags.push("Managed".to_string());
        }
        if self.flags & PartitionFlags::Raw as u32 != 0 {
            flags.push("Raw".to_string());
        }
        if self.flags & PartitionFlags::Active as u32 != 0 {
            flags.push("Active".to_string());
        }
        if self.flags & PartitionFlags::OEFI as u32 != 0 {
            flags.push("OEFI".to_string());
        }

        flags
    }
}

pub enum PartitionFlags {
    /// Constains an OS
    OS = 0x000001,
    /// Contains a bootloader
    Bootloader = 0x000002,
    /// Has POSIX permissions
    PosixPermissions = 0x000004,
    /// Is read-only
    ReadOnly = 0x000008,
    /// Is hidden
    Hidden = 0x000010,
    /// Is a system partition
    System = 0x000020,
    /// Zorya special value
    Zorya = 0x000040,
    /// Managed FS emulation
    Managed = 0x000080,
    /// Raw data partition
    Raw = 0x000100,
    /// Active partition
    Active = 0x000200,
    /// OEFI hint mask
    OEFI = 0x000c00,
}
