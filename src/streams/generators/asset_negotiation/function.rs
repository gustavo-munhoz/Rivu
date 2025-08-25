use std::io::{Error, ErrorKind};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum AssetRule {
    R1 = 1,
    R2,
    R3,
    R4,
    R5,
}

impl AssetRule {
    #[inline]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }
    #[inline]
    pub const fn as_index(self) -> usize {
        (self as u8 - 1) as usize
    }
}

impl TryFrom<u8> for AssetRule {
    type Error = Error;
    /// Maps a 1-based function_id into an [AssetRule] (R1...R5).
    /// Errors if function_id ∉ 1...=5.
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use AssetRule::*;
        match value {
            1 => Ok(R1),
            2 => Ok(R2),
            3 => Ok(R3),
            4 => Ok(R4),
            5 => Ok(R5),
            _ => Err(Error::new(
                ErrorKind::InvalidInput,
                "function must be 1..=5",
            )),
        }
    }
}
