use bitflags::bitflags;

pub mod blkptr;
pub mod sio;
pub mod stat;
pub mod spa_log;

bitflags! {
    pub struct ImportType: u8 {
        const EXISTING = 0;
        const ASSEMBLE = 1;
    }
}

bitflags! {
    pub struct Mode: u8 {
        const UNINT = 0;
        const READ = 1;
        const WRITE = 2;
    }
}

bitflags! {
    pub struct AutoTrim: u8 {
        const OFF = 0;
        const ON = 1;
        const DEFAULT = Self::OFF.bits;
    }
}

bitflags! {
    pub struct TrimType: u8 {
        const MANUAL = 0;
        const AUTO = 1;
        const SIMPLE = 2;
    }
}

bitflags! {
    pub struct SpaAsync: u32 {
        const CONFIG_UPDATE = 0x01;
        const REMOVE = 0x02;
        const PROBE = 0x04;
        const RESILVER_DONE = 0x08;
        const RESILVER = 0x10;
        const AUTOEXPAND = 0x20;
        const REMOVE_DONE = 0x40;
        const REMOVE_STOP = 0x80;
        const INITIALIZE_RESTART = 0x100;
        const TRIM_RESTART = 0x200;
        const AUTOTRIM_RESTART = 0x400;
        const L2CACHE_REBUILD = 0x800;
        const L2CACHE_TRIM = 0x1000;
        const BEBUILD_DONE = 0x2000;
    }
}

bitflags! {
    pub struct ConfigUpdate: u8 {
        const POOL = 0;
        const VDEVS = 1;
    }
}

bitflags! {
    pub struct SCL: u32 {
        const NONE = 0x00;
        const CONFIG = 0x01;
        const STATE = 0x02;
        const L2ARC = 0x04;
        const ALLOC = 0x08;
        const SIO = 0x10;
        const FREE = 0x20;
        const VDEV = 0x40;
        const LOCKS = 7;
        const ALL = (1 << Self::LOCKS.bits) - 1;
        const STATE_ALL = (Self::STATE.bits | Self::L2ARC.bits | Self::SIO.bits);
    }
}
