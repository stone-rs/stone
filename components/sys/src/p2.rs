pub trait P2Ext {
    type Output;

    fn p2align(&self, align: Self::Output) -> Self::Output;

    fn p2cross(&self, y: Self::Output, align: Self::Output) -> bool;

    fn p2roundup(&self, align: Self::Output) -> Self::Output;

    fn p2boundary(&self, len: Self::Output, align: Self::Output) -> bool;

    fn p2phase(&self, align: Self::Output) -> Self::Output;

    fn p2nphase(&self, align: Self::Output) -> Self::Output;

    fn lsp2(&self) -> bool;

    fn is_p2aligned(&self, align: Self::Output) -> bool;
}

macro_rules! p2_ext_wrapper {
    ($name:ty) => {
        impl P2Ext for $name {
            type Output = $name;

            fn p2align(&self, b: Self::Output) -> Self::Output {
                self & b.wrapping_mul(-1)
            }

            fn p2cross(&self, y: Self::Output, align: Self::Output) -> bool {
                (self ^ y) > (align - 1)
            }

            fn p2roundup(&self, align: Self::Output) -> Self::Output {
                ((self - 1) | (align - 1)) + 1
            }

            fn p2boundary(&self, len: Self::Output, align: Self::Output) -> bool {
                self ^ (self + len - 1) > (align) - 1
            }

            fn p2phase(&self, align: Self::Output) -> Self::Output {
                self & (align - 1)
            }

            fn p2nphase(&self, align: Self::Output) -> Self::Output {
                self.wrapping_mul(-1) & (align - 1)
            }

            fn lsp2(&self) -> bool {
                self & (self - 1) == 0
            }

            fn is_p2aligned(&self, align: Self::Output) -> bool {
                self & (align - 1) == 0
            }
        }
    };
}

macro_rules! p2u_ext_wrapper {
    ($name:ty) => {
        impl P2Ext for $name {
            type Output = $name;

            fn p2align(&self, b: Self::Output) -> Self::Output {
                ((*self as i128) & (b as i128).wrapping_mul(-1)) as $name
            }

            fn p2cross(&self, y: Self::Output, align: Self::Output) -> bool {
                (self ^ y) > (align - 1)
            }

            fn p2roundup(&self, align: Self::Output) -> Self::Output {
                ((self - 1) | (align - 1)) + 1
            }

            fn p2boundary(&self, len: Self::Output, align: Self::Output) -> bool {
                self ^ (self + len - 1) > (align) - 1
            }

            fn p2phase(&self, align: Self::Output) -> Self::Output {
                self & (align - 1)
            }

            fn p2nphase(&self, align: Self::Output) -> Self::Output {
                ((*self as i128).wrapping_mul(-1) & (align as i128 - 1)) as $name
            }

            fn lsp2(&self) -> bool {
                self & (self - 1) == 0
            }

            fn is_p2aligned(&self, align: Self::Output) -> bool {
                self & (align - 1) == 0
            }
        }
    };
}

p2_ext_wrapper!(i8);
p2_ext_wrapper!(i16);
p2_ext_wrapper!(i32);
p2_ext_wrapper!(i64);
p2_ext_wrapper!(i128);
p2u_ext_wrapper!(u8);
p2u_ext_wrapper!(u16);
p2u_ext_wrapper!(u32);
p2u_ext_wrapper!(u64);
p2u_ext_wrapper!(u128);
p2u_ext_wrapper!(usize);
