use std::io::Error;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AgrawalFunction {
    F1 = 1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
}

impl AgrawalFunction {
    #[inline]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }
    #[inline]
    pub const fn as_index(self) -> usize {
        (self as u8 - 1) as usize
    }
}

impl TryFrom<u8> for AgrawalFunction {
    type Error = Error;
    fn try_from(v: u8) -> Result<Self, Self::Error> {
        use AgrawalFunction::*;
        Ok(match v {
            1 => F1,
            2 => F2,
            3 => F3,
            4 => F4,
            5 => F5,
            6 => F6,
            7 => F7,
            8 => F8,
            9 => F9,
            10 => F10,
            _ => {
                return Err(Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "function must be 1..=10",
                ));
            }
        })
    }
}
