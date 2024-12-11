use core::{fmt::Display, ops::Deref};

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct Uuid([u8; 16]);

impl Uuid {
    pub fn validate(&self) -> bool {
        self.0 != [0u8; 16]
    }
}

impl Deref for Uuid {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&[u8]> for Uuid {
    fn from(value: &[u8]) -> Self {
        assert_eq!(value.len(), 16);
        let mut uuid = [0u8; 16];
        uuid.copy_from_slice(value);
        Self(uuid)
    }
}

impl Display for Uuid {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{:x}{:x}{:x}{:x}-{:x}{:x}-{:x}{:x}-{:x}{:x}-{:x}{:x}{:x}{:x}{:x}{:x}",
            self.0[3],
            self.0[2],
            self.0[1],
            self.0[0],
            self.0[5],
            self.0[4],
            self.0[7],
            self.0[6],
            self.0[8],
            self.0[9],
            self.0[10],
            self.0[11],
            self.0[12],
            self.0[13],
            self.0[14],
            self.0[15]
        )
    }
}
