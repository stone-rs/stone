use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum SIOChecksum {
    INHERIT,
    ON,
    OFF,
    LABEL,
    GANG_HEADER,
    SILOG,
    FLETCHER_2,
    FLETCHER_4,
    SHA256,
    SILOG2,
    NOPARITY,
    SHA512,
    SKEIN,
    FUNCTIONS,
}
