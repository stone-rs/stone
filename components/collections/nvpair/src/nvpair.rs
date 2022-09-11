use std::mem;

use crate::utils::{align, to_bytes};

#[warn(non_camel_case_types)]
#[derive(Debug, Clone)]
pub enum DataType {
    DONTCARE = -1,
    UNKNOWN = 0,
    BOOLEAN,
    BYTE,
    INT16,
    UINT16,
    INT32,
    UINT32,
    INT64,
    UINT64,
    STRING,
    BYTE_ARRAY,
    INT16_ARRAY,
    UINT16_ARRAY,
    INT32_ARRAY,
    UINT32_ARRAY,
    INT64_ARRAY,
    UINT64_ARRAY,
    STRING_ARRAY,
    HRTIME,
    NVLIST,
    NVLIST_ARRAY,
    BOOLEAN_VALUE,
    INT8,
    UINT8,
    BOOLEAN_ARRAY,
    INT8_ARRAY,
    UINT8_ARRAY,
    DOUBLE,
}

#[derive(Debug, Clone)]
pub struct Nvpair {
    /// size of this nvpair
    size: i32,

    /// length if name string
    name_sz: i16,

    /// not used
    reserve: i16,

    /// number of elements for array types
    value_elem: i32,

    /// type of value
    data_type: DataType,
}

impl Nvpair {
    #[inline]
    pub fn new(data_type: DataType) -> Self {
        Nvpair {
            size: 0,
            name_sz: 0,
            reserve: 0,
            value_elem: 0,
            data_type,
        }
    }

    #[inline]
    pub fn get_size(&self) -> i32 {
        self.size
    }

    #[inline]
    pub fn name(&self) -> String {
        let ptr = self as *const Nvpair as usize + mem::size_of_val(self);
        String::from_utf8_lossy(&to_bytes(ptr)).to_string()
    }

    #[inline]
    pub fn get_type(&self) -> &DataType {
        &self.data_type
    }

    #[inline]
    pub fn nelem(&self) -> i32 {
        self.value_elem
    }

    #[inline]
    pub fn value(&self) -> String {
        let size_of = mem::size_of_val(self);
        let ptr = self as *const Nvpair as usize + size_of;
        let size = to_bytes(ptr + align(size_of) + (self.name_sz as usize));
        String::from_utf8_lossy(&size).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        let p1 = Nvpair::new(DataType::BOOLEAN_ARRAY);
        let p2 = Nvpair::new(DataType::BOOLEAN_ARRAY);
        println!("{:?} {:?}", p1.name(), p2.name());
        assert_eq!(p1.name(), p2.name());
    }

    #[test]
    fn test_value() {
        let p1 = Nvpair::new(DataType::BOOLEAN_ARRAY);
        let p2 = Nvpair::new(DataType::BOOLEAN_ARRAY);
        println!("p1.value={:?} p2.value={:?}", p1.value(), p2.value());
        assert_eq!(p1.value(), p2.value())
    }
}
