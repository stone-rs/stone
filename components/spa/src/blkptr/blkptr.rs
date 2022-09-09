use super::{
    checksum::SIOChksum, HOST_BYTEORDER, SPA_ASIZEBITS, SPA_COMPRESSBITS, SPA_DVAS_PER_BP,
    SPA_MINBLOCKSHIFT, SPA_VDEVBITS,
};
use crate::sio;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use sys::BitOptExt;

#[derive(Debug, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum EmbeddedType {
    Data,
    Reserved, // Reserved for Delphix byteswap feature.
    Redacted,
    Types,
}

/// All SPA data is represented by 128-bit data virtual addresses (DVAs).
/// The members of the Dva should be considered opaque outside the SPA.
#[derive(Debug, Clone)]
pub struct Dva {
    dva_word: [u64; 2],
}

impl Dva {
    #[inline]
    pub fn new() -> Self {
        Dva {
            dva_word: [0u64; 2],
        }
    }

    #[inline]
    pub fn get_asize(&self) -> u64 {
        self.dva_word[0].bf_get_sb(0, SPA_ASIZEBITS, SPA_MINBLOCKSHIFT, 0)
    }

    #[inline]
    pub fn set_asize(&mut self, size: u64) {
        self.dva_word[0].bf_set_sb(0, SPA_ASIZEBITS, SPA_MINBLOCKSHIFT, 0, size);
    }

    #[inline]
    pub fn get_grid(&self) -> u64 {
        self.dva_word[0].bf_get(24, 8)
    }

    #[inline]
    pub fn set_grid(&mut self, grid: u64) {
        self.dva_word[0].bf_set(24, 8, grid)
    }

    #[inline]
    pub fn get_vdev(&self) -> u64 {
        self.dva_word[0].bf_get(32, SPA_VDEVBITS)
    }

    #[inline]
    pub fn set_vdev(&mut self, x: u64) {
        self.dva_word[0].bf_set(32, SPA_VDEVBITS, x)
    }

    #[inline]
    pub fn get_offset(&self) -> u64 {
        self.dva_word[1].bf_get_sb(0, 63, SPA_MINBLOCKSHIFT, 0)
    }

    #[inline]
    pub fn set_offset(&mut self, offset: u64) {
        self.dva_word[1].bf_set_sb(0, 63, SPA_MINBLOCKSHIFT, 0, offset)
    }

    #[inline]
    pub fn get_gang(&self) -> u64 {
        self.dva_word[1].bf_get_sb(0, 63, SPA_MINBLOCKSHIFT, 0)
    }

    #[inline]
    pub fn set_gang(&mut self, x: u64) {
        self.dva_word[1].bf_set_sb(0, 63, SPA_MINBLOCKSHIFT, 0, x)
    }

    #[inline]
    pub fn is_valid(&self) -> bool {
        self.get_asize() != 0
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        (self.dva_word[0] == 0) && (self.dva_word[1] == 0)
    }
}

impl PartialEq for Dva {
    fn eq(&self, other: &Self) -> bool {
        (self.dva_word[1] == other.dva_word[1]) && (self.dva_word[0] == other.dva_word[0])
    }
}

/// Some checksums/hashes need a 256-bit initialization salt. This salt is kept
/// secret and is suitable for use in MAC algorithms as the key.
pub struct SIOCheckSumSalt {
    pub zcs_bytes: [u8; 32],
}

impl SIOCheckSumSalt {
    #[inline]
    pub fn new() -> Self {
        SIOCheckSumSalt { zcs_bytes: [0; 32] }
    }
}

// Each block is described by its DVAs, time of birth, checksum, etc.
// The word-by-word, bit-by-bit layout of the blkptr is as follows:
//
//	64	56	48	40	32	24	16	8	0
//	+-------+-------+-------+-------+-------+-------+-------+-------+
// 0	|  pad  |	  vdev1         | GRID  |	  ASIZE		|
//	+-------+-------+-------+-------+-------+-------+-------+-------+
// 1	|G|			 offset1				|
//	+-------+-------+-------+-------+-------+-------+-------+-------+
// 2	|  pad  |	  vdev2         | GRID  |	  ASIZE		|
//	+-------+-------+-------+-------+-------+-------+-------+-------+
// 3	|G|			 offset2				|
//	+-------+-------+-------+-------+-------+-------+-------+-------+
// 4	|  pad  |	  vdev3         | GRID  |	  ASIZE		|
//	+-------+-------+-------+-------+-------+-------+-------+-------+
// 5	|G|			 offset3				|
//	+-------+-------+-------+-------+-------+-------+-------+-------+
// 6	|BDX|lvl| type	| cksum |E| comp|    PSIZE	|     LSIZE	|
//	+-------+-------+-------+-------+-------+-------+-------+-------+
// 7	|			padding					|
//	+-------+-------+-------+-------+-------+-------+-------+-------+
// 8	|			padding					|
//	+-------+-------+-------+-------+-------+-------+-------+-------+
// 9	|			physical birth txg			|
//	+-------+-------+-------+-------+-------+-------+-------+-------+
// a	|			logical birth txg			|
//	+-------+-------+-------+-------+-------+-------+-------+-------+
// b	|			fill count				|
//	+-------+-------+-------+-------+-------+-------+-------+-------+
// c	|			checksum[0]				|
//	+-------+-------+-------+-------+-------+-------+-------+-------+
// d	|			checksum[1]				|
//	+-------+-------+-------+-------+-------+-------+-------+-------+
// e	|			checksum[2]				|
//	+-------+-------+-------+-------+-------+-------+-------+-------+
// f	|			checksum[3]				|
//	+-------+-------+-------+-------+-------+-------+-------+-------+
//
// Legend:
//
// vdev		virtual device ID
// offset	offset into virtual device
// LSIZE	logical size
// PSIZE	physical size (after compression)
// ASIZE	allocated size (including RAID-Z parity and gang block headers)
// GRID		RAID-Z layout information (reserved for future use)
// cksum	checksum function
// comp		compression function
// G		gang block indicator
// B		byteorder (endianness)
// D		dedup
// X		encryption
// E		blkptr_t contains embedded data (see below)
// lvl		level of indirection
// type		DMU object type
// phys birth	txg when dva[0] was written; zero if same as logical birth txg
//              note that typically all the dva's would be written in this
//              txg, but they could be different if they were moved by
//              device removal.
// log. birth	transaction group in which the block was logically born
// fill count	number of non-zero blocks under this bp
// checksum[4]	256-bit checksum of the data this bp describes
///

// The blkptr_t's of encrypted blocks also need to store the encryption
// parameters so that the block can be decrypted. This layout is as follows:
//
//	64	56	48	40	32	24	16	8	0
//	+-------+-------+-------+-------+-------+-------+-------+-------+
// 0	|		vdev1		| GRID  |	  ASIZE		|
//	+-------+-------+-------+-------+-------+-------+-------+-------+
// 1	|G|			 offset1				|
//	+-------+-------+-------+-------+-------+-------+-------+-------+
// 2	|		vdev2		| GRID  |	  ASIZE		|
//	+-------+-------+-------+-------+-------+-------+-------+-------+
// 3	|G|			 offset2				|
//	+-------+-------+-------+-------+-------+-------+-------+-------+
// 4	|			salt					|
//	+-------+-------+-------+-------+-------+-------+-------+-------+
// 5	|			IV1					|
//	+-------+-------+-------+-------+-------+-------+-------+-------+
// 6	|BDX|lvl| type	| cksum |E| comp|    PSIZE	|     LSIZE	|
//	+-------+-------+-------+-------+-------+-------+-------+-------+
// 7	|			padding					|
//	+-------+-------+-------+-------+-------+-------+-------+-------+
// 8	|			padding					|
//	+-------+-------+-------+-------+-------+-------+-------+-------+
// 9	|			physical birth txg			|
//	+-------+-------+-------+-------+-------+-------+-------+-------+
// a	|			logical birth txg			|
//	+-------+-------+-------+-------+-------+-------+-------+-------+
// b	|		IV2		|	    fill count		|
//	+-------+-------+-------+-------+-------+-------+-------+-------+
// c	|			checksum[0]				|
//	+-------+-------+-------+-------+-------+-------+-------+-------+
// d	|			checksum[1]				|
//	+-------+-------+-------+-------+-------+-------+-------+-------+
// e	|			MAC[0]					|
//	+-------+-------+-------+-------+-------+-------+-------+-------+
// f	|			MAC[1]					|
//	+-------+-------+-------+-------+-------+-------+-------+-------+
//
// Legend:
//
// salt		Salt for generating encryption keys
// IV1		First 64 bits of encryption IV
// X		Block requires encryption handling (set to 1)
// E		blkptr_t contains embedded data (set to 0, see below)
// fill count	number of non-zero blocks under this bp (truncated to 32 bits)
// IV2		Last 32 bits of encryption IV
// checksum[2]	128-bit checksum of the data this bp describes
// MAC[2]	128-bit message authentication code for this data
//
// The X bit being set indicates that this block is one of 3 types. If this is
// a level 0 block with an encrypted object type, the block is encrypted
// (see BP_IS_ENCRYPTED()). If this is a level 0 block with an unencrypted
// object type, this block is authenticated with an HMAC (see
// BP_IS_AUTHENTICATED()). Otherwise (if level > 0), this bp will use the MAC
// words to store a checksum-of-MACs from the level below (see
// BP_HAS_INDIRECT_MAC_CKSUM()). For convenience in the code, BP_IS_PROTECTED()
// refers to both encrypted and authenticated blocks and BP_USES_CRYPT()
// refers to any of these 3 kinds of blocks.
//
// The additional encryption parameters are the salt, IV, and MAC which are
// explained in greater detail in the block comment at the top of zio_crypt.c.
// The MAC occupies half of the checksum space since it serves a very similar
// purpose: to prevent data corruption on disk. The only functional difference
// is that the checksum is used to detect on-disk corruption whether or not the
// encryption key is loaded and the MAC provides additional protection against
// malicious disk tampering. We use the 3rd DVA to store the salt and first
// 64 bits of the IV. As a result encrypted blocks can only have 2 copies
// maximum instead of the normal 3. The last 32 bits of the IV are stored in
// the upper bits of what is usually the fill count. Note that only blocks at
// level 0 or -2 are ever encrypted, which allows us to guarantee that these
// 32 bits are not trampled over by other code (see zio_crypt.c for details).
// The salt and IV are not used for authenticated bps or bps with an indirect
// MAC checksum, so these blocks can utilize all 3 DVAs and the full 64 bits
// for the fill count.

// "Embedded" blkptr_t's don't actually point to a block, instead they
// have a data payload embedded in the blkptr_t itself.  See the comment
// in blkptr.c for more details.
//
// The blkptr_t is laid out as follows:
//
//	64	56	48	40	32	24	16	8	0
//	+-------+-------+-------+-------+-------+-------+-------+-------+
// 0	|      payload                                                  |
// 1	|      payload                                                  |
// 2	|      payload                                                  |
// 3	|      payload                                                  |
// 4	|      payload                                                  |
// 5	|      payload                                                  |
//	+-------+-------+-------+-------+-------+-------+-------+-------+
// 6	|BDX|lvl| type	| etype |E| comp| PSIZE|              LSIZE	|
//	+-------+-------+-------+-------+-------+-------+-------+-------+
// 7	|      payload                                                  |
// 8	|      payload                                                  |
// 9	|      payload                                                  |
//	+-------+-------+-------+-------+-------+-------+-------+-------+
// a	|			logical birth txg			|
//	+-------+-------+-------+-------+-------+-------+-------+-------+
// b	|      payload                                                  |
// c	|      payload                                                  |
// d	|      payload                                                  |
// e	|      payload                                                  |
// f	|      payload                                                  |
//	+-------+-------+-------+-------+-------+-------+-------+-------+
//
// Legend:
//
// payload		contains the embedded data
// B (byteorder)	byteorder (endianness)
// D (dedup)		padding (set to zero)
// X			encryption (set to zero)
// E (embedded)		set to one
// lvl			indirection level
// type			DMU object type
// etype		how to interpret embedded data (BP_EMBEDDED_TYPE_*)
// comp			compression function of payload
// PSIZE		size of payload after compression, in bytes
// LSIZE		logical size of payload, in bytes
//			note that 25 bits is enough to store the largest
//			"normal" BP's LSIZE (2^16// 2^9) in bytes
// log. birth		transaction group in which the block was logically born
//
// Note that LSIZE and PSIZE are stored in bytes, whereas for non-embedded
// bp's they are stored in units of SPA_MINBLOCKSHIFT.
// Generally, the generic BP_GET_*() macros can be used on embedded BP's.
// The B, D, X, lvl, type, and comp fields are stored the same as with normal
// BP's so the BP_SET_* macros can be used with them.  etype, PSIZE, LSIZE must
// be set with the BPE_SET_* macros.  BP_SET_EMBEDDED() should be called before
// other macros, as they assert that they are only used on BP's of the correct
// "embedded-ness". Encrypted blkptr_t's cannot be embedded because they use
// the payload space for encryption parameters (see the comment above on
// how encryption parameters are stored).

#[derive(Debug, Clone)]
pub enum BPEmbeddedType {
    Data,
    Reserved,
    Redacted,
    Types,
}

/// A block is a hole when it has either 1) never been written to, or
/// 2) is zero-filled. In both cases, ZFS can return all zeroes for all reads
/// without physically allocating disk space. Holes are represented in the
/// blkptr_t structure by zeroed blk_dva. Correct checking for holes is
/// done through the BP_IS_HOLE macro. For holes, the logical size, level,
/// DMU object type, and birth times are all also stored for holes that
/// were written to at some point (i.e. were punched after having been filled).
#[derive(Debug, Clone)]
pub struct Blkptr {
    /// Data Virtual Address
    pub blk_dva: Vec<Dva>,

    /// size, compression, type, etcd
    pub blk_prop: u64,

    /// Extra spec for the future
    pub blk_pad: [u64; 2],

    /// txg when block was allocated
    pub blk_phys_birth: u64,

    /// transaction group at birth
    pub blk_birth: u64,

    /// fill count
    pub blk_fill: u64,

    /// 256-bit checksum
    pub blk_cksum: SIOChksum,
}

impl Blkptr {
    #[inline]
    pub fn new() -> Self {
        let blk_dvas = {
            let mut v = Vec::with_capacity(SPA_DVAS_PER_BP);
            for _ in 0..SPA_DVAS_PER_BP {
                v.push(Dva::new());
            }
            v
        };

        Blkptr {
            blk_dva: blk_dvas,
            blk_prop: 0,
            blk_pad: [0; 2],
            blk_phys_birth: 0,
            blk_birth: 0,
            blk_fill: 0,
            blk_cksum: SIOChksum::new(),
        }
    }

    #[inline]
    pub fn get_etype(&self) -> u64 {
        self.blk_prop.bf_get(40, 8)
    }

    #[inline]
    pub fn set_etype(&mut self, val: u64) {
        self.blk_prop.bf_set(40, 8, val)
    }

    #[inline]
    pub fn get_lsize(&self) -> u64 {
        self.blk_prop.bf_get_sb(0, 25, 0, 1)
    }

    #[inline]
    pub fn set_lsize(&mut self, lsize: u64) {
        self.blk_prop.bf_set_sb(0, 25, 0, 1, lsize);
    }

    #[inline]
    pub fn get_psize(&self) -> u64 {
        self.blk_prop.bf_get_sb(25, 7, 0, 1)
    }

    #[inline]
    pub fn set_psize(&mut self, psize: u64) {
        self.blk_prop.bf_set_sb(25, 7, 0, 1, psize);
    }

    #[inline]
    pub fn is_payload_word(&self, wp: u64) -> bool {
        (wp != self.blk_prop) && (wp != self.blk_birth)
    }

    #[inline]
    pub fn get_compress(&self) -> u64 {
        self.blk_prop.bf_get(32, SPA_COMPRESSBITS)
    }

    #[inline]
    pub fn set_compress(&mut self, val: u64) {
        self.blk_prop.bf_set(32, SPA_COMPRESSBITS, val)
    }

    #[inline]
    pub fn is_embedded(&self) -> bool {
        self.blk_prop.bf_get(39, 1) > 0
    }

    #[inline]
    pub fn set_embeded(&mut self, val: u64) {
        self.blk_prop.bf_set(39, 1, val)
    }

    #[inline]
    pub fn get_checksum(&self) -> sio::SIOChecksum {
        if self.is_embedded() {
            return sio::SIOChecksum::OFF;
        }
        let cks: sio::SIOChecksum = (self.blk_prop.bf_get(40, 8) as u8).try_into().unwrap();
        cks
    }

    #[inline]
    pub fn set_checksum(&mut self, cks: sio::SIOChecksum) {
        assert!(self.is_embedded());
        let val: u8 = cks.into();
        self.blk_prop.bf_set(39, 1, val as u64);
    }

    #[inline]
    pub fn get_type(&self) -> u8 {
        self.blk_prop.bf_get(48, 8) as u8
    }

    #[inline]
    pub fn set_type(&mut self, val: u8) {
        self.blk_prop.bf_set(48, 8, val as u64)
    }

    #[inline]
    pub fn get_level(&self) -> u8 {
        self.blk_prop.bf_get(56, 5) as u8
    }

    #[inline]
    pub fn set_level(&mut self, val: u8) {
        self.blk_prop.bf_set(56, 5, val as u64)
    }

    /// encrypted, authenticated, and MAC cksum bps use the same bit
    #[inline]
    pub fn get_user_crypt(&self) -> bool {
        self.blk_prop.bf_get(61, 1) != 0
    }

    #[inline]
    pub fn set_user_crypt(&mut self, val: bool) {
        self.blk_prop.bf_set(61, 1, val as u64)
    }

    #[inline]
    pub fn is_encrypted(&self) -> bool {
        self.get_user_crypt() && self.get_level() <= 0
        //  && !DMU_OT_IS_ENCRYPTED(self.get_type())
    }

    #[inline]
    pub fn is_authenticated(&self) -> bool {
        self.get_user_crypt() && (self.get_type() <= 0)
        // && !DMU_OT_IS_ENCRYPTED(self.get_type())
    }

    #[inline]
    pub fn has_indirect_mac_chksum(&self) -> bool {
        self.get_user_crypt() && (self.get_type() > 0)
    }

    #[inline]
    pub fn is_protected(&self) -> bool {
        self.is_encrypted() || self.is_authenticated()
    }

    #[inline]
    pub fn get_dedup(&self) -> bool {
        self.blk_prop.bf_get(62, 1) != 0
    }

    #[inline]
    pub fn set_dedup(&mut self, val: bool) {
        self.blk_prop.bf_set(62, 1, val as u64)
    }

    #[inline]
    pub fn get_byteorder(&self) -> u8 {
        self.blk_prop.bf_get(63, 1) as u8
    }

    #[inline]
    pub fn set_byteorder(&mut self, val: u8) {
        self.blk_prop.bf_set(63, 1, val as u64)
    }

    #[inline]
    pub fn get_free(&mut self) -> bool {
        self.blk_fill.bf_get(0, 1) != 0
    }

    #[inline]
    pub fn set_free(&mut self, val: bool) {
        self.blk_fill.bf_set(0, 1, val as u64);
    }

    #[inline]
    pub fn physical_birth(&self) -> u64 {
        if self.is_embedded() {
            return 0;
        }
        if self.blk_phys_birth > 0 {
            return self.blk_phys_birth;
        }
        self.blk_birth
    }

    #[inline]
    pub fn set_birth(&mut self, logical: u64, physical: u64) {
        assert!(!self.is_embedded());
        self.blk_birth = logical;
        self.blk_phys_birth = physical;
    }

    #[inline]
    pub fn get_fill(&self) -> u64 {
        if self.is_encrypted() {
            return self.blk_fill.bf_get(0, 32);
        }
        if self.is_embedded() {
            return 1;
        }
        self.blk_fill
    }

    #[inline]
    pub fn set_fill(&mut self, fill: u64) {
        if self.is_encrypted() {
            self.blk_fill.bf_set(0, 32, fill);
        } else {
            self.blk_fill = fill;
        }
    }

    #[inline]
    pub fn get_iv2(&self) -> u64 {
        assert!(self.is_encrypted());
        self.blk_fill.bf_get(32, 32)
    }

    #[inline]
    pub fn set_iv2(&mut self, iv2: u64) {
        assert!(self.is_encrypted());
        self.blk_fill.bf_set(32, 32, iv2)
    }

    #[inline]
    pub fn is_metadata(&self) -> bool {
        self.get_level() > 0
        // || (DMU_OT_IS_METADATA(BP_GET_TYPE(bp)))
    }

    #[inline]
    pub fn get_asize(&self) -> u64 {
        let mut asize = 0u64;
        if self.is_embedded() {
            return asize;
        }
        asize += self.blk_dva[0].get_asize();
        asize += self.blk_dva[1].get_asize();
        if !self.is_encrypted() {
            asize += self.blk_dva[2].get_asize();
        }
        asize
    }

    #[inline]
    pub fn get_ucsize(&self) -> u64 {
        if self.is_metadata() {
            return self.get_psize();
        }
        self.get_lsize()
    }

    #[inline]
    pub fn get_ndvas(&self) -> i32 {
        let mut n = 0;
        if self.is_embedded() {
            return n;
        }
        if self.blk_dva[0].get_asize() > 0 {
            n += 1;
        }
        if self.blk_dva[1].get_asize() > 0 {
            n += 1;
        }
        if !self.is_encrypted() && self.blk_dva[2].get_asize() > 0 {
            n += 1;
        }

        n
    }

    #[inline]
    pub fn count_gang(&self) -> u64 {
        let mut n = 0u64;
        if self.is_embedded() {
            return n;
        }

        n += self.blk_dva[0].get_gang();
        n += self.blk_dva[1].get_gang();
        if !self.is_encrypted() {
            n += self.blk_dva[2].get_gang();
        }

        n
    }

    #[inline]
    pub fn get_identify(&self) -> &Dva {
        assert!(!self.is_embedded());
        &self.blk_dva[0]
    }

    #[inline]
    pub fn is_gang(&self) -> bool {
        if self.is_embedded() {
            return false;
        }
        self.get_identify().get_gang() != 0
    }

    #[inline]
    pub fn is_hole(&self) -> bool {
        !self.is_embedded() && self.get_identify().is_empty()
    }

    #[inline]
    pub fn set_redacted(&mut self) {
        self.set_embeded(1);
        let t: u8 = EmbeddedType::Redacted.into();
        self.set_etype(t as u64);
    }

    #[inline]
    pub fn is_redacted(&self) -> bool {
        if !self.is_embedded() {
            return false;
        }
        let t: u8 = EmbeddedType::Redacted.into();
        t == self.get_etype() as u8
    }

    // assumes no block compression
    #[inline]
    pub fn is_raidz(&self) -> bool {
        self.blk_dva[0].get_asize() > self.get_psize()
    }

    #[inline]
    pub fn zero(&mut self) {
        self.blk_dva[0].dva_word[0] = 0;
        self.blk_dva[0].dva_word[1] = 0;
        self.blk_dva[1].dva_word[0] = 0;
        self.blk_dva[1].dva_word[1] = 0;
        self.blk_dva[2].dva_word[0] = 0;
        self.blk_dva[2].dva_word[1] = 0;
        self.blk_prop = 0;
        self.blk_pad[0] = 0;
        self.blk_pad[1] = 0;
        self.blk_phys_birth = 0;
        self.blk_birth = 0;
        self.blk_fill = 0;
        self.blk_cksum.set_checksum(0, 0, 0, 0);
    }

    #[inline]
    pub fn should_byteswap(&self) -> bool {
        self.get_byteorder() as u8 != HOST_BYTEORDER
    }
}

impl PartialEq for Blkptr {
    fn eq(&self, other: &Blkptr) -> bool {
        (self.physical_birth() == other.physical_birth())
            && (self.blk_birth == other.blk_birth)
            && (self.blk_dva[0] == other.blk_dva[0])
            && (self.blk_dva[1] == other.blk_dva[1])
            && (self.blk_dva[2] == other.blk_dva[2])
    }
}

#[cfg(test)]
mod tests {
    use super::Dva;

    #[test]
    pub fn dva_asize() {
        let mut d = Dva::new();
        let s: u64 = 1024 * 1024 * 4;
        d.set_asize(s);
        assert_eq!(d.get_asize(), s);

        let grid: u64 = 1;
        d.set_grid(grid);
        assert_eq!(d.get_grid(), grid);

        let vdev: u64 = 50;
        d.set_vdev(vdev);
        assert_eq!(d.get_vdev(), vdev);

        let offset: u64 = 4096;
        d.set_offset(offset);
        assert_eq!(d.get_offset(), offset);
    }
}
