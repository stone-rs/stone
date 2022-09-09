use bitflags::bitflags;

bitflags! {
    pub struct TxgState: u8 {
        const BIRTH = 0;
        const OPEN = 1;
        const QUIESCED = 2;
        const WAIT_FOR_SYNC = 3;
        const SYNCED = 4;
        const COMMITTED = 5;
    }
}
