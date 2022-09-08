/// General-purpose 32-bit and 64-bit bitfield encodings.
/// #define	BF32_DECODE(x, low, len)	P2PHASE((x) >> (low), 1U << (len))
#[macro_export]
macro_rules! bf32_decode {
    ($x:expr, $low:expr, $len:expr) => {
        crate::p2phase!($x >> $low, 1i32 << $len)
    };
}

/// #define	BF64_DECODE(x, low, len)	P2PHASE((x) >> (low), 1ULL << (len))
#[macro_export]
macro_rules! bf64_decode {
    ($x:expr, $low:expr, $len:expr) => {
        crate::p2phase!($x >> $low, 1i64 << $len)
    };
}
/// #define	BF32_ENCODE(x, low, len)	(P2PHASE((x), 1U << (len)) << (low))
#[macro_export]
macro_rules! bf32_encode {
    ($x:expr, $low:expr, $len:expr) => {
        crate::p2phase!($x, 1i32 << $len) << $low
    };
}

/// #define	BF64_ENCODE(x, low, len)	(P2PHASE((x), 1ULL << (len)) << (low))
#[macro_export]
macro_rules! bf64_encode {
    ($x:expr, $low:expr, $len:expr) => {
        crate::p2phase!($x, 1i64 << $len) << $low
    };
}

/// #define	BF32_GET(x, low, len)		BF32_DECODE(x, low, len)
#[macro_export]
macro_rules! bf32_get {
    ($x:expr, $low:expr, $len:expr) => {
        bf32_decode!($x, $low, $len)
    };
}

/// #define	BF64_GET(x, low, len)		BF64_DECODE(x, low, len)
#[macro_export]
macro_rules! bf64_get {
    ($x:expr, $low:expr, $len:expr) => {
        bf64_decode!($x, $low, $len)
    };
}

// #define	BF32_SET(x, low, len, val) do { \
// 	ASSERT3U(val, <, 1U << (len)); \
// 	ASSERT3U(low + len, <=, 32); \
// 	(x) ^= BF32_ENCODE((x >> low) ^ (val), low, len); \
// _NOTE(CONSTCOND) } while (0)
#[macro_export]
macro_rules! bf32_set {
    ($x:expr, $low:expr, $len:expr, $val:expr) => {{
        assert!($val < (1i32 << $len));
        assert!(($low + $len) <= 32);
        $x ^ bf32_encode!(($x >> $low) ^ $val, $low, $len)
    }};
}

// #define	BF64_SET(x, low, len, val) do { \
// 	ASSERT3U(val, <, 1ULL << (len)); \
// 	ASSERT3U(low + len, <=, 64); \
// 	((x) ^= BF64_ENCODE((x >> low) ^ (val), low, len)); \
// _NOTE(CONSTCOND) } while (0)
#[macro_export]
macro_rules! bf64_set {
    ($x:expr, $low:expr, $len:expr, $val:expr) => {{
        assert!($val < (1i64 << $len));
        assert!(($low + $len) <= 64);
        $x ^ bf64_encode!(($x >> $low) ^ $val, $low, $len)
    }};
}

// #define	BF32_GET_SB(x, low, len, shift, bias)	\
// 	((BF32_GET(x, low, len) + (bias)) << (shift))
#[macro_export]
macro_rules! bf32_get_sb {
    ($x:expr, $low:expr, $len:expr, $shift:expr, $bias:expr) => {
        (bf32_get!($x, $low, $len) + $bias) << $shift
    };
}

// #define	BF64_GET_SB(x, low, len, shift, bias)	\
// 	((BF64_GET(x, low, len) + (bias)) << (shift))
#[macro_export]
macro_rules! bf64_get_sb {
    ($x:expr, $low:expr, $len:expr, $shift:expr, $bias:expr) => {
        (bf64_get!($x, $low, $len) + $bias) << $shift
    };
}

// We use ASSERT3U instead of ASSERT in these macros to prevent a lint error in
// the case where val is a constant.  We can't fix ASSERT because it's used as
// an expression in several places in the kernel; as a result, changing it to
// the do{} while() syntax to allow us to _NOTE the CONSTCOND is not an option.
//
// #define	BF32_SET_SB(x, low, len, shift, bias, val) do { \
// 	ASSERT3U(IS_P2ALIGNED(val, 1U << shift), !=, B_FALSE); \
// 	ASSERT3S((val) >> (shift), >=, bias); \
// 	BF32_SET(x, low, len, ((val) >> (shift)) - (bias)); \
// _NOTE(CONSTCOND) } while (0)
macro_rules! bf32_set_sb {
    ($x:expr, $low:expr, $len:expr, $shift:expr, $bias:expr, $val:expr) => {{
        assert!(crate::is_p2aligned!($val, 1 << $shift) != false);
        assert!(($val >> $shift) >= $bias);
        bf32_set!($x, $low, $len, ($val >> $shift) - $bias)
    }};
}

// #define	BF64_SET_SB(x, low, len, shift, bias, val) do { \
// 	ASSERT3U(IS_P2ALIGNED(val, 1ULL << shift), !=, B_FALSE); \
// 	ASSERT3S((val) >> (shift), >=, bias); \
// 	BF64_SET(x, low, len, ((val) >> (shift)) - (bias)); \
// _NOTE(CONSTCOND) } while (0)
#[macro_export]
macro_rules! bf64_set_sb {
    ($x:expr, $low:expr, $len:expr, $shift:expr, $bias:expr, $val:expr) => {{
        assert!(crate::is_p2aligned!($val, 1 << $shift) != false);
        assert!(($val >> $shift) >= $bias);
        bf64_set!($x, $low, $len, ($val >> $shift) - $bias)
    }};
}

#[cfg(test)]
mod tests {
    #[test]
    pub fn test_bf32_decode() {
        assert_eq!(bf32_decode(0x111, 2, 2), bf32_decode!(0x111, 2, 2));
        println!("{:?}", bf32_decode!(0x111, 2, 2));
    }

    fn bf32_decode(x: i32, low: i32, len: i32) -> i32 {
        crate::p2phase!(x >> low, 1i32 << len)
    }

    #[test]
    pub fn test_bf64_decode() {
        assert_eq!(bf64_decode(0x111, 2, 2), bf64_decode!(0x111, 2, 2));
        println!("{:?}", bf64_decode!(0x111, 2, 2));
    }

    fn bf64_decode(x: i64, low: i64, len: i64) -> i64 {
        crate::p2phase!(x >> low, 1i64 << len)
    }

    #[test]
    pub fn test_bf32_encode() {
        assert_eq!(bf32_encode(0x111, 2, 2), bf32_encode!(0x111, 2, 2));
        println!("{:?}", bf32_encode!(0x111, 2, 2));
    }

    fn bf32_encode(x: i32, low: i32, len: i32) -> i32 {
        crate::p2phase!(x, 1i32 << len) << low
    }

    #[test]
    pub fn test_bf64_encode() {
        assert_eq!(bf64_encode(0x111, 2, 2), bf64_encode!(0x111, 2, 2));
        println!("{:?}", bf64_encode!(0x111, 2, 2));
    }

    fn bf64_encode(x: i64, low: i64, len: i64) -> i64 {
        crate::p2phase!(x, 1i64 << len) << low
    }

    #[test]
    pub fn test_bf32_set() {
        assert_eq!(bf32_set(0x111, 2, 2, 0x001), bf32_set!(0x111, 2, 2, 0x001));
        println!("{:?}", bf32_set!(0x111, 2, 2, 0x001));
    }

    fn bf32_set(x: i32, low: i32, len: i32, val: i32) -> i32 {
        x ^ bf32_encode!((x >> low) ^ val, low, len)
    }

    #[test]
    pub fn test_bf64_set() {
        assert_eq!(bf64_set(0x111, 2, 2, 0x001), bf64_set!(0x111, 2, 2, 0x001));
        println!("{:?}", bf64_set!(0x111, 2, 2, 0x001));
    }

    fn bf64_set(x: i64, low: i64, len: i64, val: i64) -> i64 {
        x ^ bf64_encode!((x >> low) ^ val, low, len)
    }

    #[test]
    pub fn test_bf32_get_sb() {
        assert_eq!(
            bf32_get_sb(0x111, 2, 2, 0x001, 9),
            bf32_get_sb!(0x111, 2, 2, 0x001, 9)
        );
        println!("{:?}", bf32_get_sb!(0x111, 2, 2, 0x001, 9));
    }

    fn bf32_get_sb(x: i32, low: i32, len: i32, shift: i32, bias: i32) -> i32 {
        (bf32_get!(x, low, len) + bias) << shift
    }

    #[test]
    pub fn test_bf64_get_sb() {
        assert_eq!(
            bf64_get_sb(0x111, 2, 2, 0x001, 9),
            bf64_get_sb!(0x111, 2, 2, 0x001, 9)
        );
        println!("{:?}", bf64_get_sb!(0x111, 2, 2, 0x001, 9));
    }

    fn bf64_get_sb(x: i64, low: i64, len: i64, shift: i64, bias: i64) -> i64 {
        (bf64_get!(x, low, len) + bias) << shift
    }

    #[test]
    pub fn test_bf32_set_sb() {
        assert_eq!(
            bf32_set_sb(0x111, 2, 2, 0, 1, 1),
            bf32_set_sb!(0x111, 2, 2, 0, 1, 1)
        );
        println!("{:?}", bf32_set_sb!(0x111, 2, 2, 0, 1, 1));
    }

    fn bf32_set_sb(x: i32, low: i32, len: i32, shift: i32, bias: i32, val: i32) -> i32 {
        bf32_set!(x, low, len, (val >> shift) - bias)
    }

    #[test]
    pub fn test_bf64_set_sb() {
        assert_eq!(
            bf64_set_sb(0x111, 2, 2, 0, 1, 1),
            bf64_set_sb!(0x111, 2, 2, 0, 1, 1)
        );
        println!("{:?}", bf64_set_sb!(0x111, 2, 2, 0, 1, 1));
    }

    fn bf64_set_sb(x: i64, low: i64, len: i64, shift: i64, bias: i64, val: i64) -> i64 {
        bf64_set!(x, low, len, (val >> shift) - bias)
    }
}
