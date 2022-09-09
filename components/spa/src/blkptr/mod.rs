pub mod blkptr;
pub mod checksum;

use std::alloc::Layout;

use blkptr::*;
use checksum::*;

// We currently support block sizes from 512 bytes to 16MB.
// The benefits of larger blocks, and thus larger IO, need to be weighed
// against the cost of COWing a giant block to modify one byte, and the
// large latency of reading or writing a large block.
//
// Note that although blocks up to 16MB are supported, the recordsize
// property can not be set larger than zfs_max_recordsize (default 1MB).
// See the comment near zfs_max_recordsize in dsl_dataset.c for details.
//
// Note that although the LSIZE field of the blkptr_t can store sizes up
// to 32MB, the dnode's dn_datablkszsec can only store sizes up to
// 32MB - 512 bytes.  Therefore, we limit SPA_MAXBLOCKSIZE to 16MB.
const SPA_MINBLOCKSHIFT: u64 = 9;
const SPA_OLD_MAXBLOCKSHIFT: u64 = 17;
const SPA_MAXBLOCKSHIFT: u64 = 24;
const SPA_MINBLOCKSIZE: u64 = 1 << SPA_MINBLOCKSHIFT;
const SPA_OLD_MAXBLOCKSIZE: u64 = 1 << SPA_OLD_MAXBLOCKSHIFT;

// Alignment Shift (ashift) is an immutable, internal top-level vdev property
// which can only be set at vdev creation time. Physical writes are always done
// according to it, which makes 2^ashift the smallest possible IO on a vdev.
//
// We currently allow values ranging from 512 bytes (2^9 = 512) to 64 KiB
// (2^16 = 65,536).
const ASHIFT_MIN: usize = 9;
const ASHIFT_MAX: usize = 16;

// Size of block to hold the configuration data (a packed nvlist)
const SPA_CONFIG_BLOCKSIZE: usize = 1 << 14;

// The DVA size encodings for LSIZE and PSIZE support blocks up to 32MB.
// The ASIZE encoding should be at least 64 times larger (6 more bits)
// to support up to 4-way RAID-Z mirror mode with worst-case gang block
// overhead, three DVAs per bp, plus one more bit in case we do anything
// else that expands the ASIZE.
const SPA_LSIZEBITS: u64 = 16; /* LSIZE up to 32M (2^16// 512)	*/
const SPA_PSIZEBITS: u64 = 16; /* PSIZE up to 32M (2^16// 512)	*/
const SPA_ASIZEBITS: u64 = 24; /* ASIZE up to 64 times larger	*/

const SPA_COMPRESSBITS: u64 = 7;
const SPA_VDEVBITS: u64 = 24;
const SPA_COMPRESSMASK: u64 = (1 << SPA_COMPRESSBITS) - 1;

const BPE_NUM_WORDS: usize = 14;
const BPE_PAYLOAD_SIZE: usize = BPE_NUM_WORDS * Layout::new::<u64>().align();

// blkptr_t is 128 bytes
const SPA_BLKPTRSHIFT: u64 = 7;
// Number of DVAs in a bp
const SPA_DVAS_PER_BP: usize = 3;
// min vdevs to update during sync
const SPA_SYNC_MIN_VDEVS: u64 = 3;

#[cfg(target_endian = "little")]
const HOST_BYTEORDER: u8 = 0;
#[cfg(target_endian = "big")]
const HOST_BYTEORDER: u8 = 1;
