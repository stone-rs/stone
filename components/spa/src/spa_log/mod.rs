use bitflags::bitflags;

bitflags! {
    pub struct LogState: u8 {
        const UNKNOWN = 0; // unknown log state
        const MISSING = 1; // missing log(s)
        const CLEAR = 2; // clear the log(s)
        const GOOD = 3; // log(s) are good
    }
}
