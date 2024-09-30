#![no_std]
mod err;
mod hdr;
mod mbr;
mod partition;
mod uuid;

use core::mem::MaybeUninit;

extern crate alloc;

use alloc::{boxed::Box, vec::Vec};
use byteorder::{ByteOrder, LittleEndian};
pub use err::*;
pub use hdr::*;

pub use mbr::*;
pub use partition::*;
pub use uuid::Uuid;

#[derive(Debug)]
pub struct GptLayout {
    protective_mbr: Box<MaybeUninit<ProtectiveMBR>>,
    primary_header: Box<MaybeUninit<Header>>,
    partitions: Vec<(Partition, usize)>,
    backup_partitions: Vec<(Partition, usize)>,
    backup_header: Box<MaybeUninit<Header>>,
}

impl GptLayout {
    pub fn new() -> Self {
        Self {
            protective_mbr: Box::new_uninit(),
            primary_header: Box::new_uninit(),
            partitions: Vec::with_capacity(PARTITION_LBA_SIZE),
            backup_partitions: Vec::with_capacity(PARTITION_LBA_SIZE),
            backup_header: Box::new_uninit(),
        }
    }

    pub fn init_primary_header(&mut self, blk: &[u8]) -> Result<(), HeaderError> {
        let header = Header::deserialize(&blk)?;
        let init = MaybeUninit::new(header);
        self.primary_header = Box::new(init);
        Ok(())
    }

    pub fn init_backup_header(&mut self, blk: &[u8]) -> Result<(), HeaderError> {
        let header = Header::deserialize(&blk)?;
        let init = MaybeUninit::new(header);
        self.backup_header = Box::new(init);
        Ok(())
    }

    pub fn init_protective_mbr(&mut self, blk: &[u8]) -> Result<(), MBRError> {
        let mbr = ProtectiveMBR::deserialize(&blk)?;
        let init = MaybeUninit::new(mbr);
        self.protective_mbr = Box::new(init);
        Ok(())
    }

    pub fn init_partitions(&mut self, blk: &[u8], entry_index: usize) {
        let part_num = blk.len() / PARTITION_LBA_SIZE;
        let part_index = (entry_index - 1) * part_num;
        (0..part_num).for_each(|index| {
            let start = index * PARTITION_LBA_SIZE;
            let end = start + PARTITION_LBA_SIZE;
            if let Some(part) = Partition::deserialize(&blk[start..end]) {
                self.partitions.push((part, part_index + index + 1));
            }
        });
    }

    pub fn init_backup_partitions(&mut self, blk: &[u8], entry_index: usize) {
        let part_num = blk.len() / PARTITION_LBA_SIZE;
        let part_index = (entry_index - 1) * part_num;
        (0..part_num).for_each(|index| {
            let start = index * PARTITION_LBA_SIZE;
            let end = start + PARTITION_LBA_SIZE;
            if let Some(part) = Partition::deserialize(&blk[start..end]) {
                self.backup_partitions.push((part, part_index + index + 1));
            }
        });
    }
}

impl GptLayout {
    pub fn protective_mbr(&self) -> &ProtectiveMBR {
        unsafe { self.protective_mbr.as_ref().assume_init_ref() }
    }

    pub fn primary_header(&self) -> &Header {
        unsafe { self.primary_header.as_ref().assume_init_ref() }
    }
    pub fn partition(&self, part_index: usize) -> Option<&Partition> {
        assert!(part_index < MIN_PARTITION_NUM);
        let mut find_index = PARTITION_LBA_SIZE;
        for (i, (_, part_i)) in self.partitions.iter().enumerate() {
            if *part_i == part_index {
                find_index = i;
                break;
            }
        }
        if find_index != PARTITION_LBA_SIZE {
            Some(&self.partitions.get(find_index).unwrap().0)
        } else {
            None
        }
    }
    pub fn backup_partition(&self, part_index: usize) -> Option<&Partition> {
        assert!(part_index < MIN_PARTITION_NUM);
        let mut find_index = PARTITION_LBA_SIZE;
        for (i, (_, part_i)) in self.backup_partitions.iter().enumerate() {
            if *part_i == part_index {
                find_index = i;
                break;
            }
        }
        if find_index != PARTITION_LBA_SIZE {
            Some(&self.backup_partitions.get(find_index).unwrap().0)
        } else {
            None
        }
    }
    pub fn backup_header(&self) -> &Header {
        unsafe { self.backup_header.as_ref().assume_init_ref() }
    }
}

impl GptLayout {
    pub fn protective_mbr_mut(&mut self) -> &mut ProtectiveMBR {
        unsafe { self.protective_mbr.as_mut().assume_init_mut() }
    }
    pub fn primary_header_mut(&mut self) -> &mut Header {
        unsafe { self.primary_header.as_mut().assume_init_mut() }
    }
    pub fn partition_mut(&mut self, part_index: usize) -> Option<&mut Partition> {
        assert!(part_index < MIN_PARTITION_NUM);
        let mut find_index = PARTITION_LBA_SIZE;
        for (i, (_, part_i)) in self.partitions.iter().enumerate() {
            if *part_i == part_index {
                find_index = i;
                break;
            }
        }
        if find_index != PARTITION_LBA_SIZE {
            Some(&mut self.partitions.get_mut(find_index).unwrap().0)
        } else {
            None
        }
    }
    pub fn backup_partition_mut(&mut self, part_index: usize) -> Option<&mut Partition> {
        assert!(part_index < MIN_PARTITION_NUM);
        let mut find_index = PARTITION_LBA_SIZE;
        for (i, (_, part_i)) in self.backup_partitions.iter().enumerate() {
            if *part_i == part_index {
                find_index = i;
                break;
            }
        }
        if find_index != PARTITION_LBA_SIZE {
            Some(&mut self.backup_partitions.get_mut(find_index).unwrap().0)
        } else {
            None
        }
    }
    pub fn backup_header_mut(&mut self) -> &mut Header {
        unsafe { self.backup_header.as_mut().assume_init_mut() }
    }
}

fn write_to_bytes<const SIZE: usize>(val: u64, bytes: &mut [u8], start: usize) {
    let mut bts = [0u8; SIZE];
    match SIZE {
        2 => LittleEndian::write_u16(&mut bts, val as u16),
        4 => LittleEndian::write_u32(&mut bts, val as u32),
        8 => LittleEndian::write_u64(&mut bts, val),
        _ => {}
    }
    for index in 0..SIZE {
        bytes[start + index] = bts[index];
    }
}

fn copy_bytes<T: ToU8>(src: &[T], dst: &mut [u8], start: usize, size: usize) {
    (0..size).for_each(|index| dst[index + start] = src[index].as_u8());
}

trait ToU8 {
    fn as_u8(&self) -> u8;
}

impl ToU8 for char {
    fn as_u8(&self) -> u8 {
        *self as u8
    }
}

impl ToU8 for u8 {
    fn as_u8(&self) -> u8 {
        *self
    }
}
