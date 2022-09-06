use std::ops::{Bound, RangeBounds};

/// Range with only an start bound, excluded
pub struct RangeFromExcluded<T> {
    pub start: T,
}

impl<T> RangeFromExcluded<T> {
    pub const fn new(start: T) -> RangeFromExcluded<T> {
        RangeFromExcluded { start }
    }
}

impl<T> RangeBounds<T> for RangeFromExcluded<T> {
    fn start_bound(&self) -> std::ops::Bound<&T> {
        Bound::Excluded(&self.start)
    }

    fn end_bound(&self) -> std::ops::Bound<&T> {
        Bound::Unbounded
    }
}

/// Range with an excluded start bound and included end bound.
pub struct RangeFromExcludedToIncluded<T> {
    pub start: T,
    pub end: T,
}

impl<T> RangeFromExcludedToIncluded<T> {
    pub const fn new(start: T, end: T) -> RangeFromExcludedToIncluded<T> {
        RangeFromExcludedToIncluded { start, end }
    }
}

impl<T> RangeBounds<T> for RangeFromExcludedToIncluded<T> {
    fn start_bound(&self) -> std::ops::Bound<&T> {
        Bound::Excluded(&self.start)
    }

    fn end_bound(&self) -> std::ops::Bound<&T> {
        Bound::Included(&self.end)
    }
}

/// Range where both bounds are excluded.
pub struct RangeFromExcludedTo<T> {
    pub start: T,
    pub end: T,
}

impl<T> RangeFromExcludedTo<T> {
    pub const fn new(start: T, end: T) -> RangeFromExcludedTo<T> {
        RangeFromExcludedTo { start, end }
    }
}

impl<T> RangeBounds<T> for RangeFromExcludedTo<T> {
    fn start_bound(&self) -> Bound<&T> {
        Bound::Excluded(&self.start)
    }

    fn end_bound(&self) -> Bound<&T> {
        Bound::Excluded(&self.end)
    }
}
