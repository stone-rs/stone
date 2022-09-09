use super::P2Ext;

pub trait BitOptExt {
    type Output;

    fn bf_decode(&self, low: Self::Output, len: Self::Output) -> Self::Output;

    fn bf_encode(&self, low: Self::Output, len: Self::Output) -> Self::Output;

    fn bf_get(&self, low: Self::Output, len: Self::Output) -> Self::Output;

    fn bf_set(&mut self, low: Self::Output, len: Self::Output, val: Self::Output);

    fn bf_get_sb(
        &self,
        low: Self::Output,
        len: Self::Output,
        shift: Self::Output,
        bias: Self::Output,
    ) -> Self::Output;

    fn bf_set_sb(
        &mut self,
        low: Self::Output,
        len: Self::Output,
        shift: Self::Output,
        bias: Self::Output,
        val: Self::Output,
    );
}

macro_rules! bit_opt_ext_wrapper {
    ($name:ty, $bit:expr) => {
        impl BitOptExt for $name {
            type Output = $name;

            fn bf_decode(&self, low: Self::Output, len: Self::Output) -> Self::Output {
                (self >> low).p2phase((1 as $name) << len)
            }

            fn bf_encode(&self, low: Self::Output, len: Self::Output) -> Self::Output {
                self.p2phase((1 as $name) << len) << low
            }

            fn bf_get(&self, low: Self::Output, len: Self::Output) -> Self::Output {
                self.bf_decode(low, len)
            }

            fn bf_set(&mut self, low: Self::Output, len: Self::Output, val: Self::Output) {
                assert!(val < ((1 as $name) << len));
                assert!((low + len) <= $bit);
                let x = (*self >> low) ^ val;
                *self ^= x.bf_encode(low, len);
            }

            fn bf_get_sb(
                &self,
                low: Self::Output,
                len: Self::Output,
                shift: Self::Output,
                bias: Self::Output,
            ) -> Self::Output {
                (self.bf_get(low, len) + bias) << shift
            }

            fn bf_set_sb(
                &mut self,
                low: Self::Output,
                len: Self::Output,
                shift: Self::Output,
                bias: Self::Output,
                val: Self::Output,
            ) {
                assert!(val.is_p2aligned((1 as $name) << shift) != false);
                assert!((val >> shift) >= bias);
                self.bf_set(low, len, (val >> shift) - bias);
            }
        }
    };
}

bit_opt_ext_wrapper!(i32, 32);
bit_opt_ext_wrapper!(i64, 64);
bit_opt_ext_wrapper!(u64, 64);
bit_opt_ext_wrapper!(usize, 64);
