/// Compatibility macros/typedefs needed for Solaris -> Linux port
/// NOTE: from sysmacors.h
#[macro_export]
macro_rules! p2align {
    ($a:expr, $align:expr) => {
        $a & -($align)
    };
}

#[macro_export]
macro_rules! p2cross {
    ($x:expr, $y:expr, $align:expr) => {
        ($x ^ $y) > ($align - 1)
    };
}

#[macro_export]
macro_rules! p2roundup {
    ($x:expr, $align:expr) => {
        (($x - 1) | ($align - 1)) + 1
    };
}

#[macro_export]
macro_rules! p2boundary {
    ($off:expr, $len:expr, $align:expr) => {
        $off ^ ($off + $len - 1) > ($align) - 1
    };
}

#[macro_export]
macro_rules! p2phase {
    ($x:expr, $align:expr) => {
        $x & ($align - 1)
    };
}

#[macro_export]
macro_rules! p2nphase {
    ($x:expr, $align:expr) => {
        -($x) & ($align - 1)
    };
}

#[macro_export]
macro_rules! isp2 {
    ($x:expr) => {
        $x & (&x - 1) == 0
    };
}

#[macro_export]
macro_rules! is_p2aligned {
    ($x:expr, $align:expr) => {
        $x & ($align - 1) == 0
    };
}

#[cfg(test)]
pub mod test {
    #[test]
    pub fn test_p2align() {
        assert_eq!(p2align(0x001, 2), p2align!(0x001, 2));
        println!("{:?}", p2align!(0x001, 2));
    }

    #[test]
    pub fn test_p2cross() {
        assert_eq!(p2cross(0x001, 0x010, 3), p2cross!(0x001, 0x010, 3));
        println!("{:?}", p2cross(0x001, 0x010, 3));
    }

    fn p2align(a: i32, b: i32) -> i32 {
        a & -(b)
    }

    fn p2cross(x: i32, y: i32, align: i32) -> bool {
        (x ^ y) > (align - 1)
    }

    fn p2roundup(x: i32, align: i32) -> i32 {
        ((x - 1) | (align - 1)) + 1
    }

    #[test]
    pub fn test_p2roundup() {
        assert_eq!(p2roundup(0x011, 2), p2roundup!(0x011, 2));
        println!("{:?}", p2roundup!(0x011, 2));
    }

    fn p2boundary(off: i32, len: i32, align: i32) -> bool {
        ((off) ^ ((off) + (len) - 1)) > (align) - 1
    }

    #[test]
    pub fn test_p2boundary() {
        assert_eq!(p2boundary(0x011, 2, 2), p2boundary!(0x011, 2, 2));
        println!("{:?}", p2boundary!(0x011, 2, 2));
    }

    fn p2phase(x: i32, align: i32) -> i32 {
        x & (align - 1)
    }

    #[test]
    pub fn test_p2phase() {
        assert_eq!(p2phase!(0x011, 2), p2phase(0x011, 2));
        println!("{:?}", p2phase!(0x011, 2));
    }

    fn p2nphase(x: i32, align: i32) -> i32 {
        -(x) & (align - 1)
    }

    #[test]
    pub fn test_p2nphase() {
        assert_eq!(p2nphase!(0x011, 2), p2nphase(0x011, 2));
        println!("{:?}", p2nphase!(0x011, 2));
    }
}
