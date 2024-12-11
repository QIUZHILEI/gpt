use core::fmt::Display;

use super::{copy_bytes, write_to_bytes, Uuid};
use alloc::string::String;
use byteorder::{ByteOrder, LittleEndian};
pub const PARTITION_LBA_SIZE: usize = 128;
pub const MIN_PARTITION_NUM: usize = 128;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Partition {
    /// GUID of the partition type.
    pub part_type_guid: Uuid,
    /// UUID of the partition.
    pub part_guid: Uuid,
    /// First LBA of the partition.
    pub start_lba: u64,
    /// Last LBA of the partition.
    pub end_lba: u64,
    /// Partition flags.
    pub attrs: u64,
    /// Partition name.
    pub name: PartitionName,
}

impl Default for Partition {
    fn default() -> Self {
        Self {
            part_type_guid: Default::default(),
            part_guid: Default::default(),
            start_lba: Default::default(),
            end_lba: Default::default(),
            attrs: Default::default(),
            name: PartitionName([0u8; 72]),
        }
    }
}

impl Partition {
    pub fn deserialize(blk: &[u8]) -> Option<Self> {
        let part_type_guid = Uuid::from(&blk[0..16]);
        let part_guid = Uuid::from(&blk[16..32]);
        let start_lba = LittleEndian::read_u64(&blk[32..40]);
        let end_lba = LittleEndian::read_u64(&blk[40..48]);
        let attrs = LittleEndian::read_u64(&blk[48..56]);
        let name = PartitionName::from(&blk[56..PARTITION_LBA_SIZE]);
        if part_type_guid.validate() {
            Some(Self {
                part_type_guid,
                part_guid,
                start_lba,
                end_lba,
                attrs,
                name,
            })
        } else {
            None
        }
    }

    pub fn serialize(&self) -> [u8; PARTITION_LBA_SIZE] {
        let mut bytes = [0u8; PARTITION_LBA_SIZE];
        copy_bytes(&self.part_type_guid, &mut bytes, 0, 16);
        copy_bytes(&self.part_guid, &mut bytes, 16, 16);
        write_to_bytes::<8>(self.start_lba, &mut bytes, 32);
        write_to_bytes::<8>(self.end_lba, &mut bytes, 40);
        write_to_bytes::<8>(self.attrs, &mut bytes, 48);
        copy_bytes(&self.name.0, &mut bytes, 56, 72);
        bytes
    }
}

impl Display for Partition {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Partition Entry{{\n
            \tpartition type guid:{},\n
            \tpartition guid: {},\n
            \tstart_lba: {},\n
            \tend_lba: {},\n
            \tname: {},\n}}",
            self.part_type_guid, self.part_guid, self.start_lba, self.end_lba, self.name
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartitionName([u8; 72]);

impl From<&[u8]> for PartitionName {
    fn from(value: &[u8]) -> Self {
        let mut name = [0u8; 72];
        name.copy_from_slice(value);
        Self(name)
    }
}

impl Display for PartitionName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let name = &self.0;
        let name =
            unsafe { String::from_raw_parts(name as *const u8 as *mut u8, name.len(), name.len()) };
        write!(f, "{}", name)
    }
}
