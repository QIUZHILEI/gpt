use core::fmt::Display;

use super::{copy_bytes, write_to_bytes, HeaderError, Uuid};
use byteorder::{ByteOrder, LittleEndian};
pub const PRIMARY_HEADER_LBA: usize = 1;
pub const GPT_SIGNATURE: [char; 8] = ['E', 'F', 'I', ' ', 'P', 'A', 'R', 'T'];
/// Header describing a GPT disk.
#[derive(Clone, Debug, Default)]
pub struct Header {
    /// GPT header magic signature, hardcoded to "EFI PART".
    pub signature: [char; 8], // Offset  0. "EFI PART", 45h 46h 49h 20h 50h 41h 52h 54h
    /// major, minor
    pub revision: (u16, u16), // Offset  8
    /// little endian
    pub header_size: u32, // Offset 12
    /// CRC32 of the header, will be incorrect after changing something until the
    /// header get's written
    pub crc32: u32, // Offset 16
    /// must be 0
    pub reserved: u32, // Offset 20
    /// For main header, 1
    pub my_lba: u64, // Offset 24
    /// LBA for backup header
    pub backup_lba: u64, // Offset 32
    /// First usable LBA for partitions (primary table last LBA + 1)
    pub first_usable: u64, // Offset 40
    /// Last usable LBA (secondary partition table first LBA - 1)
    pub last_usable: u64, // Offset 48
    /// UUID of the disk
    pub disk_guid: Uuid, // Offset 56
    /// Starting LBA of partition entries
    pub part_start: u64, // Offset 72
    /// Number of partition entries
    pub num_parts: u32, // Offset 80
    /// Size of a partition entry, usually 128
    pub part_size: u32, // Offset 84
    /// CRC32 of the partition table, will be incorrect after changing something until the
    /// header get's written
    pub crc32_parts: u32, // Offset 88
}

impl Header {
    pub fn deserialize(blk: &[u8]) -> Result<Self, HeaderError> {
        let _ = check_signature(&blk[0..8])?;
        let crc32 = LittleEndian::read_u32(&blk[16..20]);
        let header = Self {
            signature: GPT_SIGNATURE,
            revision: {
                let minor = LittleEndian::read_u16(&blk[8..10]);
                let major = LittleEndian::read_u16(&blk[10..12]);
                (major, minor)
            },
            header_size: LittleEndian::read_u32(&blk[12..16]),
            crc32,
            reserved: LittleEndian::read_u32(&blk[20..24]),
            my_lba: LittleEndian::read_u64(&blk[24..32]),
            backup_lba: LittleEndian::read_u64(&blk[32..40]),
            first_usable: LittleEndian::read_u64(&blk[40..48]),
            last_usable: LittleEndian::read_u64(&blk[48..56]),
            disk_guid: Uuid::from(&blk[56..72]),
            part_start: LittleEndian::read_u64(&blk[72..80]),
            num_parts: LittleEndian::read_u32(&blk[80..84]),
            part_size: LittleEndian::read_u32(&blk[84..88]),
            crc32_parts: LittleEndian::read_u32(&blk[88..92]),
        };
        Ok(header)
    }

    pub fn serialize(&self) -> [u8; size_of::<Self>()] {
        let mut bytes = [0u8; size_of::<Self>()];
        for (index, ele) in self.signature.iter().enumerate() {
            bytes[index] = *ele as u8;
        }
        write_to_bytes::<2>(self.revision.1 as u64, &mut bytes, 8);
        write_to_bytes::<2>(self.revision.0 as u64, &mut bytes, 10);
        write_to_bytes::<4>(self.header_size as u64, &mut bytes, 12);
        write_to_bytes::<4>(self.crc32 as u64, &mut bytes, 16);
        write_to_bytes::<8>(self.my_lba, &mut bytes, 24);
        write_to_bytes::<8>(self.backup_lba, &mut bytes, 32);
        write_to_bytes::<8>(self.first_usable as u64, &mut bytes, 40);
        write_to_bytes::<8>(self.last_usable as u64, &mut bytes, 48);
        copy_bytes(&self.disk_guid, &mut bytes, 56, 16);
        write_to_bytes::<8>(self.part_start, &mut bytes, 72);
        write_to_bytes::<4>(self.num_parts as u64, &mut bytes, 80);
        write_to_bytes::<4>(self.part_size as u64, &mut bytes, 84);
        write_to_bytes::<4>(self.crc32_parts as u64, &mut bytes, 88);
        bytes
    }
}

fn check_signature(sig: &[u8]) -> Result<(), HeaderError> {
    for (index, ele) in sig.iter().enumerate() {
        let item = *ele as char;
        if !item.eq(&GPT_SIGNATURE[index]) {
            return Err(HeaderError::InvalidGptSignature);
        }
    }
    Ok(())
}

impl Display for Header {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Header{{\n 
            \trevision: {}.{},\n
            \theader_size: {},\n
            \tcrc32: {},\n
            \tmy_lba: {},\n
            \tbackup_lba: {},\n
            \tfirst_usable: {},\n
            \tlast_usable: {},\n
            \tdisk_guid: {},\n
            \tpart_start: {},\n
            \tnum_parts: {},\n
            \tpart_size: {},\n
            \tcrc32_parts: {},\n}}",
            self.revision.0,
            self.revision.1,
            self.header_size,
            self.crc32,
            self.my_lba,
            self.backup_lba,
            self.first_usable,
            self.last_usable,
            self.disk_guid,
            self.part_start,
            self.num_parts,
            self.part_size,
            self.crc32_parts
        )
    }
}
