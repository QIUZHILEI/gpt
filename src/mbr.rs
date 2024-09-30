use core::fmt;

use byteorder::{ByteOrder, LittleEndian};

use super::{copy_bytes, write_to_bytes, MBRError};

pub const PROTECTIVE_MBR_LBA: usize = 0;
pub const MBR_SIGNATURE: [u8; 2] = [0x55, 0xAA];
#[derive(Clone)]
pub struct ProtectiveMBR {
    pub bootcode: [u8; 440],
    pub disk_signature: [u8; 4],
    pub unknown: u16,
    pub partitions: [PartRecord; 4],
    pub signature: [u8; 2],
}

impl Default for ProtectiveMBR {
    fn default() -> Self {
        Self {
            bootcode: [0u8; 440],
            disk_signature: Default::default(),
            unknown: Default::default(),
            partitions: Default::default(),
            signature: Default::default(),
        }
    }
}

impl fmt::Debug for ProtectiveMBR {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Protective MBR, partitions: {:#?}", self.partitions)
    }
}

impl ProtectiveMBR {
    pub fn deserialize(blk: &[u8]) -> Result<Self, MBRError> {
        let mut bootcode = [0u8; 440];
        bootcode.copy_from_slice(&blk[0..440]);
        let disk_signature = [0u8; 4];
        bootcode.copy_from_slice(&blk[440..444]);
        let unknown = LittleEndian::read_u16(&blk[444..446]);
        let partitions = [
            PartRecord::from_bytes(&blk[446..462])?,
            PartRecord::from_bytes(&blk[462..478])?,
            PartRecord::from_bytes(&blk[478..494])?,
            PartRecord::from_bytes(&blk[494..510])?,
        ];
        let signature = [blk[510], blk[511]];
        if signature != MBR_SIGNATURE {
            Err(MBRError::InvalidMBRSignature)
        } else {
            Ok(Self {
                bootcode,
                disk_signature,
                unknown,
                partitions,
                signature,
            })
        }
    }
    pub fn serialize(&self) -> [u8; size_of::<Self>()] {
        let mut bytes = [0u8; 512];
        copy_bytes(&self.bootcode, &mut bytes, 0, 440);
        copy_bytes(&self.disk_signature, &mut bytes, 440, 4);
        write_to_bytes::<2>(self.unknown as u64, &mut bytes, 444);
        self.partitions
            .iter()
            .enumerate()
            .for_each(|(index, part)| {
                copy_bytes(&part.to_bytes(), &mut bytes, index * 16 + 446, 16)
            });
        copy_bytes(&self.signature, &mut bytes, 510, 2);
        bytes
    }
}

/// A partition record, MBR-style.
#[derive(Copy, Clone, Debug, Default)]
pub struct PartRecord {
    /// Bit 7 set if partition is active (bootable)
    pub boot_indicator: u8,
    /// CHS address of partition start: 8-bit value of head in CHS address
    pub start_head: u8,
    /// CHS address of partition start: Upper 2 bits are 8th-9th bits of cylinder, lower 6 bits are sector
    pub start_sector: u8,
    /// CHS address of partition start: Lower 8 bits of cylinder
    pub start_track: u8,
    /// Partition type. See <https://www.win.tue.nl/~aeb/partitions/partition_types-1.html>
    pub os_type: u8,
    /// CHS address of partition end: 8-bit value of head in CHS address
    pub end_head: u8,
    /// CHS address of partition end: Upper 2 bits are 8th-9th bits of cylinder, lower 6 bits are sector
    pub end_sector: u8,
    /// CHS address of partition end: Lower 8 bits of cylinder
    pub end_track: u8,
    /// LBA of start of partition
    pub lb_start: u32,
    /// Number of sectors in partition
    pub lb_size: u32,
}

impl PartRecord {
    /// Create a protective Partition Record object with a specific disk size (in LB).
    pub fn new_protective(lb_size: Option<u32>) -> Self {
        let size = lb_size.unwrap_or(0xFF_FF_FF_FF);
        Self {
            boot_indicator: 0x00,
            start_head: 0x00,
            start_sector: 0x02,
            start_track: 0x00,
            os_type: 0xEE,
            end_head: 0xFF,
            end_sector: 0xFF,
            end_track: 0xFF,
            lb_start: 1,
            lb_size: size,
        }
    }

    /// Create an all-zero Partition Record.
    pub fn zero() -> Self {
        Self {
            boot_indicator: 0x00,
            start_head: 0x00,
            start_sector: 0x00,
            start_track: 0x00,
            os_type: 0x00,
            end_head: 0x00,
            end_sector: 0x00,
            end_track: 0x00,
            lb_start: 0,
            lb_size: 0,
        }
    }

    /// Parse input bytes into a Partition Record.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, MBRError> {
        if bytes.len() != 16 {
            return Err(MBRError::InvalidPartitionLength);
        }

        let pr = Self {
            boot_indicator: bytes[0],
            start_head: bytes[1],
            start_sector: bytes[2],
            start_track: bytes[3],
            os_type: bytes[4],
            end_head: bytes[5],
            end_sector: bytes[6],
            end_track: bytes[7],
            lb_start: LittleEndian::read_u32(&bytes[8..12]),
            lb_size: LittleEndian::read_u32(&bytes[12..16]),
        };

        Ok(pr)
    }

    /// Return the memory representation of this Partition Record as a byte vector.
    pub fn to_bytes(&self) -> [u8; 16] {
        let mut bytes = [0u8; 16];
        bytes[0] = self.boot_indicator;
        bytes[1] = self.start_head;
        bytes[2] = self.start_sector;
        bytes[3] = self.start_track;
        bytes[4] = self.os_type;
        bytes[5] = self.end_head;
        bytes[6] = self.end_sector;
        bytes[7] = self.end_sector;

        write_to_bytes::<4>(self.lb_start as u64, &mut bytes, 8);
        write_to_bytes::<4>(self.lb_size as u64, &mut bytes, 12);

        bytes
    }
}
