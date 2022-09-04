use crate::generic::node::{Address, Balance, Item, Node, WouldUnderflow};
use cc_traits::{SimpleCollectionMut, SimpleCollectionRef, Slab, SlabMut};
use std::{
    borrow::Borrow,
    cmp::Ordering,
    hash::{Hash, Hasher},
    iter::{DoubleEndedIterator, ExactSizeIterator, FromIterator, FusedIterator},
    marker::PhantomData,
    ops::{Bound, Index, RangeBounds},
};

mod entry;
mod ext;

pub use entry::*;
pub use ext::*;

/// Knuth order of the B-Trees.
///
/// Must be at least 4.
pub const M: usize = 8;

/// A map based on a B-Tree.
///
/// This offers an alternative over the standard implementation of B-Trees where nodes are
/// allocated in a contiguous array of [`Node`]s, reducing the cost of tree nodes allocations.
/// In addition the crate provides advanced functions to iterate through and update the map
/// efficiently.
///
/// # Basic usage
///
/// Basic usage is similar to the map data structures offered by the standard library.
/// ```
/// use btree::BTreeMap;
///
/// // type inference lets us omit an explicit type signature (which
/// // would be `BTreeMap<&str, &str>` in this example).
/// let mut movie_reviews = BTreeMap::new();
///
/// // review some movies.
/// movie_reviews.insert("Office Space",       "Deals with real issues in the workplace.");
/// movie_reviews.insert("Pulp Fiction",       "Masterpiece.");
/// movie_reviews.insert("The Godfather",      "Very enjoyable.");
/// movie_reviews.insert("The Blues Brothers", "Eye lyked it a lot.");
///
/// // check for a specific one.
/// if !movie_reviews.contains_key("Les Misérables") {
///     println!("We've got {} reviews, but Les Misérables ain't one.",
///              movie_reviews.len());
/// }
///
/// // oops, this review has a lot of spelling mistakes, let's delete it.
/// movie_reviews.remove("The Blues Brothers");
///
/// // look up the values associated with some keys.
/// let to_find = ["Up!", "Office Space"];
/// for movie in &to_find {
///     match movie_reviews.get(movie) {
///        Some(review) => println!("{}: {}", movie, review),
///        None => println!("{} is unreviewed.", movie)
///     }
/// }
///
/// // Look up the value for a key (will panic if the key is not found).
/// println!("Movie review: {}", movie_reviews["Office Space"]);
///
/// // iterate over everything.
/// for (movie, review) in &movie_reviews {
///     println!("{}: \"{}\"", movie, review);
/// }
/// ```
///
/// # Advanced usage
///
/// ## Entry API
///
/// This crate also reproduces the Entry API defined by the standard library,
/// which allows for more complex methods of getting, setting, updating and removing keys and
/// their values:
/// ```
/// use btree::BTreeMap;
///
/// // type inference lets us omit an explicit type signature (which
/// // would be `BTreeMap<&str, u8>` in this example).
/// let mut player_stats: BTreeMap<&str, u8> = BTreeMap::new();
///
/// fn random_stat_buff() -> u8 {
///     // could actually return some random value here - let's just return
///     // some fixed value for now
///     42
/// }
///
/// // insert a key only if it doesn't already exist
/// player_stats.entry("health").or_insert(100);
///
/// // insert a key using a function that provides a new value only if it
/// // doesn't already exist
/// player_stats.entry("defence").or_insert_with(random_stat_buff);
///
/// // update a key, guarding against the key possibly not being set
/// let stat = player_stats.entry("attack").or_insert(100);
/// *stat += random_stat_buff();
/// ```
///
/// ## Mutable iterators
///
/// This type provides two iterators providing mutable references to the entries:
///   - [`IterMut`] is a double-ended iterator following the standard
///     [`std::collections::btree_map::IterMut`] implementation.
///   - [`EntriesMut`] is a single-ended iterator that allows, in addition,
///     insertion and deletion of entries at the current iterator's position in the map.
///     An example is given below.
///
/// ```
/// use btree::BTreeMap;
///
/// let mut map = BTreeMap::new();
/// map.insert("a", 1);
/// map.insert("b", 2);
/// map.insert("d", 4);
///
/// let mut entries = map.entries_mut();
/// entries.next();
/// entries.next();
/// entries.insert("c", 3); // the inserted key must preserve the order of the map.
///
/// let entries: Vec<_> = map.into_iter().collect();
/// assert_eq!(entries, vec![("a", 1), ("b", 2), ("c", 3), ("d", 4)]);
/// ```
///
/// ## Custom allocation
///
/// This data structure is built on top of a slab data structure,
/// but is agnostic of the actual slab implementation which is taken as parameter (`C`).
/// If the `slab` feature is enabled,
/// the [`slab::Slab`] implementation is used by default by reexporting
/// `BTreeMap<K, V, slab::Slab<_>>` at the root of the crate.
/// Any container implementing "slab-like" functionalities can be used.
///
/// ## Extended API
///
/// This crate provides the two traits [`BTreeExt`] and [`BTreeExtMut`] that can be imported to
/// expose low-level operations on [`BTreeMap`].
/// The extended API allows the caller to directly navigate and access the entries of the tree
/// using their [`Address`].
/// These functions are not intended to be directly called by the users,
/// but can be used to extend the data structure with new functionalities.
///
/// # Correctness
///
/// It is a logic error for a key to be modified in such a way that the key's ordering relative
/// to any other key, as determined by the [`Ord`] trait, changes while it is in the map.
/// This is normally only possible through [`Cell`](`std::cell::Cell`),
/// [`RefCell`](`std::cell::RefCell`), global state, I/O, or unsafe code.
#[derive(Clone)]
pub struct BTreeMap<K, V, C> {
    /// Allocated and free nodes.
    nodes: C,

    /// Root node id,
    root: Option<usize>,

    /// Number of items in the tree.
    len: usize,

    k: PhantomData<K>,
    v: PhantomData<V>,
}

impl<K, V, C> BTreeMap<K, V, C> {
    /// Create a new empty B-tree.
    #[inline]
    pub fn new() -> BTreeMap<K, V, C>
    where
        C: Default,
    {
        BTreeMap {
            nodes: Default::default(),
            root: None,
            len: 0,
            k: PhantomData,
            v: PhantomData,
        }
    }

    /// Returns `true` if the map contains no elements.
    ///
    /// # Example
    ///
    /// ```
    /// use btree::BTreeMap;
    ///
    /// let mut a = BTreeMap::new();
    /// assert!(a.is_empty());
    /// a.insert(1, "a");
    /// assert!(!a.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    /// Returns the number of elements in the map.
    ///
    /// # Example
    ///
    /// ```
    /// use btree::BTreeMap;
    ///
    /// let mut a = BTreeMap::new();
    /// assert_eq!(a.len(), 0);
    /// a.insert(1, "a");
    /// assert_eq!(a.len(), 1);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }
}

impl<K, V, C: SlabMut<Node<K, V>>> BTreeMap<K, V, C>
where
    C: SimpleCollectionRef,
    C: SimpleCollectionMut,
{
	/// Returns the key-value pair corresponding to the supplied key.
	///
	/// The supplied key may be any borrowed form of the map's key type, but the ordering
	/// on the borrowed form *must* match the ordering on the key type.
	///
	/// # Example
	///
	/// ```
	/// use btree_slab::BTreeMap;
	///
	/// let mut map: BTreeMap<i32, &str> = BTreeMap::new();
	/// map.insert(1, "a");
	/// assert_eq!(map.get_key_value(&1), Some((&1, &"a")));
	/// assert_eq!(map.get_key_value(&2), None);
	/// ```
    #[inline]
    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V> 
    where 
        K: Borrow<Q>,
        Q: Ord
    {
        match self.root {
            Some(id) => self.get_in(key, id),
            None => None,
        }
    }
    #[inline]
    fn try_rotate_left(
        &mut self,
        id: usize,
        deficient_child_index: usize,
        addr: &mut Address,
    ) -> bool {
        true
    }

    #[inline]
    fn try_rotate_right(
        &mut self,
        id: usize,
        deficient_child_index: usize,
        addr: &mut Address,
    ) -> bool {
        true
    }

    #[inline]
    fn merge(
        &mut self,
        id: usize,
        deficient_child_index: usize,
        mut addr: Address,
    ) -> (Balance, Address) {
        todo!()
    }
}
