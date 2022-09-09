pub mod p2;
pub mod bitops;

pub use p2::*;
pub use bitops::*;



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_i32() {
        let i: i32 = 0x011;
        println!("{:?}", i.p2align(0xff));
        println!("{:?}", i.bf_encode(4, 4).bf_decode(4, 4));
    }

    #[test]
    fn test_u32() {
        let i: u32 = 12;
        println!("{:?}", i.p2align(0xffff));
        println!("{:?}", i.p2nphase(40));
    }
}
