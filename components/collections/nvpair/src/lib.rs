use bitflags::bitflags;

pub mod nvlist;
pub mod nvpair;
pub mod utils;

bitflags! {
    /// nvlist pack encoding
    pub struct NvEncode: u8 {
        const NATIVE = 0;
        const XDR = 1;
    }
}

bitflags! {
    /// nvlist persistent unique name flags, stored in nvl_nvflags
    pub struct NvUnique: u8 {
        const NAME = 0x1;
        const NAME_TYPE = 0x2;
    }
}

/// nvlist lookup pairs related flags
const NV_FLAG_NOENTOK: u8 = 0x1;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
