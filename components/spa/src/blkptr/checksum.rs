/// Each block has a 256-bit checksum -- strong enough for cryptographic hashes.
#[derive(Debug, Clone)]
pub struct SIOChksum {
    pub zc_word: [u64; 4],
}

impl SIOChksum {
    #[inline]
    pub fn new() -> Self {
        SIOChksum { zc_word: [0; 4] }
    }

    #[inline]
    pub fn set_checksum(&mut self, w0: u64, w1: u64, w2: u64, w3: u64) {
        self.zc_word[0] = w0;
        self.zc_word[1] = w1;
        self.zc_word[2] = w2;
        self.zc_word[3] = w3;
    }

    #[inline]
    pub fn is_zero(&self) -> bool {
        self.zc_word[0] == 0 | self.zc_word[1] | self.zc_word[2] | self.zc_word[3]
    }

    #[inline]
    pub fn swap_bytes(&mut self) {
        self.zc_word[0] = self.zc_word[0].swap_bytes();
        self.zc_word[1] = self.zc_word[1].swap_bytes();
        self.zc_word[2] = self.zc_word[2].swap_bytes();
        self.zc_word[3] = self.zc_word[3].swap_bytes();
    }
}

impl PartialEq for SIOChksum {
    fn eq(&self, other: &SIOChksum) -> bool {
        (self.zc_word[0] - other.zc_word[0] == 0)
            | (self.zc_word[1] - other.zc_word[1] == 0)
            | (self.zc_word[2] - other.zc_word[2] == 0)
            | (self.zc_word[3] - other.zc_word[3] == 0)
    }
}
