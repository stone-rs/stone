pub(crate) fn align(x: usize) -> usize {
    (x + 7) & !7usize
}

pub(crate) fn align4(x: u16) -> u16 {
    (x + 3) & !3
}

pub(crate) fn to_bytes(x: usize) -> [u8; 8] {
    #[cfg(target_endian = "little")]
    {
        x.to_le_bytes()
    }
    #[cfg(target_endian = "big")]
    {
        x.to_be_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_align() {
        assert_eq!(align(3), 8);
    }

    #[test]
    fn test_align4() {
        assert_eq!(align4(2), 4);
    }
}
