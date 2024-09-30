use core::fmt;

#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
/// Errors returned when interacting with a header.
pub enum HeaderError {
    /// Invalid GPT Signature
    /// This means your trying to read a gpt header which does not exist or is invalid.
    InvalidGptSignature,
    /// Invalid CRC32 Checksum
    ///
    /// This means the header was corrupted or not fully written.
    InvalidCRC32Checksum,
    // Builder errors
    /// Get's returned when you call build on a HeaderBuilder and the backup lba field
    /// was never set
    MissingBackupLba,
    /// Get's returned when you call build on a HeaderBuilder and there isn't enough space
    /// between first_lba and backup_lba
    BackupLbaToEarly,
    /// Get's returned when you try to write to the wrong lba (example calling
    /// write_primary instead of write_backup)
    WritingToWrongLba,
    /// The Disk is to small to hold a backup header
    ToSmallForBackup,
}

impl fmt::Display for HeaderError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        use HeaderError::*;
        let desc = match self {
            InvalidGptSignature => "Invalid GPT Signature, the header does not exist or is invalid",
            InvalidCRC32Checksum => "CRC32 Checksum Mismatch, the header is corrupted",
            MissingBackupLba => "HeaderBuilder expects the field backup_lba to be set",
            BackupLbaToEarly => {
                "HeaderBuilder: there isn't enough space between first_lba and backup_lba"
            },
            WritingToWrongLba => {
                "you trying to write to the wrong lba (example calling write_primary instead of write_backup)"
            },
            ToSmallForBackup => "the disk is to small to hold a backup header"
        };
        write!(fmt, "{desc}")
    }
}

#[non_exhaustive]
#[derive(Debug)]
/// Errors returned when interacting with a Gpt Disk.
pub enum MBRError {
    /// The provided buffer does not match the expected mbr length
    InvalidMBRLength,
    /// invalid MBR signature
    InvalidMBRSignature,
    /// Invalid Partition Length != 16
    InvalidPartitionLength,
}

impl fmt::Display for MBRError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        use MBRError::*;
        let desc = match self {
            InvalidMBRLength => "The provided buffer does not match the expected mbr length",
            InvalidMBRSignature => "Invalid MBR signature",
            InvalidPartitionLength => "Invalid Partition length expected 16",
        };
        write!(fmt, "{desc}")
    }
}
