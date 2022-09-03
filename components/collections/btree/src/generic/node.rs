use std::{borrow::Borrow, cmp::Ordering, fmt};

mod addr;
pub mod internal;
mod item;
mod leaf;

pub use addr::Address;
pub use internal::Internal as InternalNode;
pub use item::Item;
pub use leaf::Leaf as LeafNode;

/// Type idenfier by a key.
///
/// This is implemented by [`Item`] and [`internal::Branch`].
pub trait Keyed {
    type Key;

    fn key(&self) -> &Self::Key;
}

/// Offset in a node.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Offset(usize);

impl Offset {
    pub fn before() -> Offset {
        Offset(usize::MAX)
    }

    pub fn is_before(&self) -> bool {
        self.0 == usize::MAX
    }

    pub fn value(&self) -> Option<usize> {
        if self.0 == usize::MAX {
            None
        } else {
            Some(self.0)
        }
    }

    pub fn unwrap(self) -> usize {
        if self.0 == usize::MAX {
            panic!("Offset out of bounds")
        } else {
            self.0
        }
    }

    pub fn decr(&mut self) {
        if self.0 == 0 {
            self.0 = usize::MAX
        } else {
            self.0 -= 1
        }
    }
}

impl PartialOrd for Offset {
    fn partial_cmp(&self, offset: &Offset) -> Option<Ordering> {
        if self.0 == usize::MAX || offset.0 == usize::MAX {
            if self.0 == usize::MAX && offset.0 == usize::MAX {
                Some(Ordering::Equal)
            } else if self.0 == usize::MAX {
                Some(Ordering::Less)
            } else {
                Some(Ordering::Greater)
            }
        } else {
            self.0.partial_cmp(&offset.0)
        }
    }
}

impl Ord for Offset {
    fn cmp(&self, offset: &Offset) -> Ordering {
        if self.0 == usize::MAX || offset.0 == usize::MAX {
            if self.0 == usize::MAX && offset.0 == usize::MAX {
                Ordering::Equal
            } else if self.0 == usize::MAX {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        } else {
            self.0.cmp(&offset.0)
        }
    }
}

impl PartialEq<usize> for Offset {
    fn eq(&self, offset: &usize) -> bool {
        self.0 != usize::MAX && self.0 == *offset
    }
}

impl From<usize> for Offset {
    fn from(offset: usize) -> Self {
        Offset(offset)
    }
}

impl fmt::Display for Offset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 == usize::MAX {
            write!(f, "-1")
        } else {
            self.0.fmt(f)
        }
    }
}

impl fmt::Debug for Offset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 == usize::MAX {
            write!(f, "-1")
        } else {
            self.0.fmt(f)
        }
    }
}

/// Node balance.
pub enum Balance {
    /// The node is balanced.
    Balanced,

    /// The node is overflowing.
    Overflow,

    /// The node is underflowing.
    /// 
    /// The boolean is `true` if the node is empty.
    Underflow(bool) 
}

/// Error returned when an operation on the node would result in an underflow.
pub struct WouldUnderflow;

/// Type of the value returned by `Node::pop_right`.
/// It includes the offset of the popped item, the item itself and the index of
/// the right child of the item if it is removed from an internal node.
pub type PoppedItem<K, V> = (Offset, Item<K, V>, Option<usize>);

pub enum Children<'a, K, V> {
	Leaf,
	Internal(Option<usize>, std::slice::Iter<'a, internal::Branch<K, V>>),
}

impl<'a, K, V> Iterator for Children<'a, K, V> {
	type Item = usize;

	#[inline]
	fn next(&mut self) -> Option<usize> {
		match self {
			Children::Leaf => None,
			Children::Internal(first, rest) => match first.take() {
				Some(child) => Some(child),
				None => rest.next().map(|branch| branch.child),
			},
		}
	}
}

pub enum ChildrenWithSeparators<'a, K, V> {
	Leaf,
	Internal(
		Option<usize>,
		Option<&'a Item<K, V>>,
		std::iter::Peekable<std::slice::Iter<'a, internal::Branch<K, V>>>,
	),
}

impl<'a, K, V> Iterator for ChildrenWithSeparators<'a, K, V> {
	type Item = (Option<&'a Item<K, V>>, usize, Option<&'a Item<K, V>>);

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		match self {
			ChildrenWithSeparators::Leaf => None,
			ChildrenWithSeparators::Internal(first, left_sep, rest) => match first.take() {
				Some(child) => {
					let right_sep = rest.peek().map(|right| &right.item);
					*left_sep = right_sep;
					Some((None, child, right_sep))
				}
				None => match rest.next() {
					Some(branch) => {
						let right_sep = rest.peek().map(|right| &right.item);
						let result = Some((*left_sep, branch.child, right_sep));
						*left_sep = right_sep;
						result
					}
					None => None,
				},
			},
		}
	}
}