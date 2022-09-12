#[derive(Debug, Clone)]
pub struct NvList {
    version: i32,
    nvflag: u32,
    nvl_priv: u64,
    flag: u32,
    pad: i32,
}

impl NvList {
    #[inline]
    pub fn new() -> Self {
        NvList {
            version: 0,
            nvflag: 0,
            nvl_priv: 0,
            flag: 0,
            pad: 0,
        }
    }

    #[inline]
    pub fn get_version(&self) -> i32 {
        self.version
    }

    #[inline]
    pub fn get_flag(&self) -> u32 {
        self.flag
    }
}
