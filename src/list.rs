//! mod `list` - lists of shared values.
//!
//! See [`List`] for more info.

use crate::arcmap::ArcMap;
use crate::shared::{Link, Shareable, Shared};
/// A list of shareable values.
///
/// Using a `List<T>` rather than a `Vec<T>` allows components which use only one or two list items
/// to get updated only when the specific list items they use are changed.
///
/// ```rust
/// # use dioxus::prelude::*;
/// use dioxus_shareables::{shareable, List, ListEntry};
///
/// shareable!(Numbers: List<usize> = [3, 5, 7].into_iter().collect());
///
/// #[allow(non_snake_case)]
/// fn ListNumbers(cx: Scope) -> Element {
///     let nums = Numbers.use_rw(&cx); // This component is updated when new items are added to or
///                                     // removed from the list, but not when the individual list
///                                     // items change.
///     let w = nums.clone();
///     cx.render(rsx! {
///         ul {
///             nums.read().iter().map(|n| rsx! { ListItem { num: n } })
///         }
///         button {
///             onclick: move |_| {
///                 let mut w = w.write();
///                 let sum = w.iter().map(|n| *n.share().read()).sum();
///                 w.push(sum)
///             },
///             "Sum"
///         }
///     })
/// }
///
/// #[allow(non_snake_case)]
/// #[inline_props]
/// fn ListItem(cx: Scope, num: ListEntry<usize>) -> Element {
///     let num = num.use_rw(&cx); // This component is updated when this specific entry in the
///                                // list is modified.
///     let w1 = num.clone();
///     let w2 = num.clone();
///     let num = num.read();
///
///     cx.render(rsx! {
///         li {
///             "{num}",
///             button { onclick: move |_| *w1.write() += 1, "+" }
///             button { onclick: move |_| *w2.write() -= 1, "-" }
///         }
///     })
/// }
///
/// ```
///
/// `List` is a [`Vec`] internally, and the methods it implements therefore get their names and
/// behavior from [`Vec`].
///
pub struct List<T: 'static + Send + Sync>(Vec<ListEntry<T>>);

#[allow(non_camel_case_types)]
pub type share_entry_w<T> = fn(ListEntry<T>) -> Shared<T, super::W>;
pub type Drain<'a, T> = std::iter::Map<std::vec::Drain<'a, ListEntry<T>>, share_entry_w<T>>;
impl<T: 'static + Send + Sync> List<T> {
    /// See [`Vec::append`]
    pub fn append(&mut self, o: &mut Self) {
        self.0.append(&mut o.0);
    }
    /// See [`Vec::capacity`]
    #[allow(clippy::must_use_candidate)]
    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }
    /// See [`Vec::clear`]
    pub fn clear(&mut self) {
        self.0.clear();
    }
    /// See [`Vec::dedup`]
    pub fn dedup(&mut self)
    where
        T: PartialEq,
    {
        self.dedup_by(PartialEq::eq);
    }
    /// See [`Vec::dedup_by`]
    pub fn dedup_by<F: FnMut(&T, &T) -> bool>(&mut self, mut f: F) {
        self.0.dedup_by(|r, s| f(&r.0.borrow(), &s.0.borrow()));
    }
    /// See [`Vec::dedup_by_key`]
    pub fn dedup_by_key<K: PartialEq, F: FnMut(&T) -> K>(&mut self, mut f: F) {
        self.0.dedup_by(|r, s| f(&r.0.borrow()) == f(&s.0.borrow()));
    }
    /// See [`Vec::drain`]
    pub fn drain<R: std::ops::RangeBounds<usize>>(&mut self, range: R) -> Drain<T>
    where
        T: 'static,
    {
        self.0.drain(range).map(|l| Shared::from_link(l.0))
    }
    /// See [`Vec::insert`]
    pub fn insert(&mut self, index: usize, element: T) {
        self.0.insert(index, ListEntry::new(element));
    }
    /// See [`Vec::is_empty`]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    /// See [`Vec::len`]
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }
    /// See [`Vec::new`]
    #[must_use]
    pub fn new() -> Self {
        Self(Vec::new())
    }
    /// See [`Vec::pop`]
    pub fn pop(&mut self) -> Option<Shared<T, super::W>> {
        self.0.pop().map(|l| Shared::from_link(l.0))
    }
    /// See [`Vec::push`]
    pub fn push(&mut self, value: T) {
        self.0.push(ListEntry::new(value));
    }
    /// See [`Vec::remove`]
    pub fn remove(&mut self, index: usize) -> Shared<T, super::W> {
        Shared::from_link(self.0.remove(index).0)
    }
    /// See [`Vec::reserve`]
    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional);
    }
    /// See [`Vec::reserve_exact`]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.0.reserve_exact(additional);
    }
    /// See [`Vec::resize`]
    pub fn resize(&mut self, new_len: usize, t: T)
    where
        T: Clone,
    {
        self.0.resize_with(new_len, || ListEntry::new(t.clone()));
    }
    /// See [`Vec::resize_with`]
    pub fn resize_with<F: FnMut() -> T>(&mut self, new_len: usize, mut f: F) {
        self.0.resize_with(new_len, || ListEntry::new(f()));
    }
    /// See [`Vec::retain`]
    pub fn retain<F: FnMut(&T) -> bool>(&mut self, mut f: F) {
        self.0.retain(|l| f(&l.0.borrow()));
    }
    /// See [`Vec::retain`]
    pub fn retain_mut<F: FnMut(&mut ListEntry<T>) -> bool>(&mut self, f: F)
    where
        T: 'static,
    {
        self.0.retain_mut(f);
    }
    /// See [`Vec::shrink_to`]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.0.shrink_to(min_capacity);
    }
    /// See [`Vec::shrink_to_fit`]
    pub fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit();
    }
    /// See [`Vec::splice`]
    pub fn splice<'a, R: std::ops::RangeBounds<usize>, I: 'a + IntoIterator<Item = T>>(
        &'a mut self,
        range: R,
        replace_with: I,
    ) -> impl 'a + Iterator<Item = Shared<T, super::W>>
    where
        T: 'static,
    {
        self.0
            .splice(range, replace_with.into_iter().map(ListEntry::new))
            .map(|l| Shared::from_link(l.0))
    }
    /// See [`Vec::split_off`]
    #[must_use = "use `.truncate()` if you don't need the other half"]
    pub fn split_off(&mut self, at: usize) -> Self {
        Self(self.0.split_off(at))
    }
    /// See [`Vec::swap_remove`]
    pub fn swap_remove(&mut self, index: usize) -> Shared<T, super::W> {
        Shared::from_link(self.0.swap_remove(index).0)
    }
    /// See [`Vec::truncate`]
    pub fn truncate(&mut self, len: usize) {
        self.0.truncate(len);
    }
    /// See [`Vec::try_reserve`]
    ///
    /// # Errors
    /// Returns an error if the capcity overflows or the allocator reports an error.
    pub fn try_reserve(
        &mut self,
        additional: usize,
    ) -> Result<(), std::collections::TryReserveError> {
        self.0.try_reserve(additional)
    }
    /// See [`Vec::try_reserve_exact`]
    ///
    /// # Errors
    /// Returns an error if the capcity overflows or the allocator reports an error.
    pub fn try_reserve_exact(
        &mut self,
        additional: usize,
    ) -> Result<(), std::collections::TryReserveError> {
        self.0.try_reserve_exact(additional)
    }
    /// See [`Vec::with_capacity`]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }
    /// See [`[_]::binary_search`]
    #[allow(clippy::missing_errors_doc)]
    pub fn binary_search(&self, x: &T) -> Result<usize, usize>
    where
        T: Ord,
    {
        self.binary_search_by(|l| x.cmp(l))
    }
    /// See [`[_]::binary_search`]
    #[allow(clippy::missing_errors_doc)]
    pub fn binary_search_by<F: FnMut(&T) -> std::cmp::Ordering>(
        &self,
        mut f: F,
    ) -> Result<usize, usize> {
        self.0.binary_search_by(|l| f(&l.0.borrow()))
    }
    /// See [`[_]::binary_search_by_key`]
    #[allow(clippy::missing_errors_doc)]
    pub fn binary_search_by_key<B: std::cmp::Ord, F: FnMut(&T) -> B>(
        &self,
        b: &B,
        mut f: F,
    ) -> Result<usize, usize> {
        self.0.binary_search_by_key(b, |l| f(&l.0.borrow()))
    }
    /// See [`[_]::contains`]
    pub fn contains(&self, x: &T) -> bool
    where
        T: PartialEq,
    {
        self.0.iter().any(|l| x == &*l.0.borrow())
    }
    /// See [`[_]::ends_with`]
    pub fn ends_with(&self, needle: &[T]) -> bool
    where
        T: PartialEq,
    {
        self.0.len() >= needle.len()
            && std::iter::zip(self.0.iter().rev(), needle.iter().rev())
                .all(|(l, x)| x == &*l.0.borrow())
    }
    /// See [`[_]::fill`]
    ///
    /// Note: This replaces items, rather than changing their value, so components which were
    /// linked to the list before will not (necessarily) update.
    pub fn fill(&mut self, t: T)
    where
        T: Clone,
    {
        self.0.fill_with(|| ListEntry::new(t.clone()));
    }
    /// See [`[_]::fill_with`]
    ///
    /// Note: This replaces items, rather than changing their value, so components which were
    /// linked to the list before will not (necessarily) update.
    pub fn fill_with<F: FnMut() -> T>(&mut self, mut f: F) {
        self.0.fill_with(|| ListEntry::new(f()));
    }
    /// See [`[_]::first`]
    #[must_use]
    pub fn first(&self) -> Option<ListEntry<T>> {
        self.0.first().cloned()
    }
    /// See [`[_]::get`]
    #[must_use]
    pub fn get(&self, index: usize) -> Option<ListEntry<T>> {
        self.0.get(index).cloned()
    }
    /// See [`[_]::get_unchecked`]
    ///
    /// # Safety
    ///   * The index must be in bounds for the slice, otherwise this method is u.b.
    #[must_use]
    pub unsafe fn get_unchecked(&self, index: usize) -> ListEntry<T> {
        self.0.get_unchecked(index).clone()
    }
    /// See [`[_]::iter`]
    #[allow(clippy::must_use_candidate)]
    pub fn iter(&self) -> <&Self as IntoIterator>::IntoIter {
        self.into_iter()
    }
    /// See [`[_]::last`]
    #[must_use]
    pub fn last(&self) -> Option<ListEntry<T>> {
        self.0.last().cloned()
    }
    /// See [`[_]::partition_point`]
    pub fn partition_point<P: FnMut(&T) -> bool>(&self, mut pred: P) -> usize {
        self.0.partition_point(|l| pred(&l.0.borrow()))
    }
    /// See [`[_]::reverse`]
    pub fn reverse(&mut self) {
        self.0.reverse();
    }
    /// See [`[_]::rotate_left`]
    pub fn rotate_left(&mut self, mid: usize) {
        self.0.rotate_left(mid);
    }
    /// See [`[_]::rotate_right`]
    pub fn rotate_right(&mut self, mid: usize) {
        self.0.rotate_right(mid);
    }
    /// See [`[_]::sort`]
    pub fn sort(&mut self)
    where
        T: Ord,
    {
        self.sort_by(Ord::cmp);
    }
    /// See [`[_]::sort_by`]
    pub fn sort_by<F: FnMut(&T, &T) -> std::cmp::Ordering>(&mut self, mut f: F) {
        self.0.sort_by(|a, b| f(&a.0.borrow(), &b.0.borrow()));
    }
    /// See [`[_]::sort_by`]
    pub fn sort_by_cached_key<U: Ord, F: FnMut(&T) -> U>(&mut self, mut f: F) {
        self.0.sort_by_cached_key(|a| f(&a.0.borrow()));
    }
    /// See [`[_]::sort_by`]
    pub fn sort_by_key<U: Ord, F: FnMut(&T) -> U>(&mut self, mut f: F) {
        self.0.sort_by_key(|a| f(&a.0.borrow()));
    }
    /// See [`[_]::sort`]
    pub fn sort_unstable(&mut self)
    where
        T: Ord,
    {
        self.sort_unstable_by(Ord::cmp);
    }
    /// See [`[_]::sort_by`]
    pub fn sort_unstable_by<F: FnMut(&T, &T) -> std::cmp::Ordering>(&mut self, mut f: F) {
        self.0
            .sort_unstable_by(|a, b| f(&a.0.borrow(), &b.0.borrow()));
    }
    /// See [`[_]::sort_by`]
    pub fn sort_unstable_by_key<U: Ord, F: FnMut(&T) -> U>(&mut self, mut f: F) {
        self.0.sort_unstable_by_key(|a| f(&a.0.borrow()));
    }
    /// See [`[_]::starts_with`]
    pub fn starts_with(&self, needle: &[T]) -> bool
    where
        T: PartialEq,
    {
        self.0.len() >= needle.len()
            && std::iter::zip(&self.0, needle).all(|(l, x)| x == &*l.0.borrow())
    }
    /// See [`[_]::swap`]
    pub fn swap(&mut self, a: usize, b: usize) {
        self.0.swap(a, b);
    }
}
impl<T: 'static + Send + Sync> Default for List<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<'a, T: 'static + Send + Sync> IntoIterator for &'a List<T> {
    type Item = ListEntry<T>;
    type IntoIter = std::iter::Cloned<std::slice::Iter<'a, ListEntry<T>>>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter().cloned()
    }
}
impl<T: 'static + Send + Sync> FromIterator<T> for List<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(iter.into_iter().map(ListEntry::new).collect())
    }
}
impl<T: 'static + Send + Sync> Extend<T> for List<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.0.extend(iter.into_iter().map(ListEntry::new));
    }
}
impl<'a, T: 'static + Send + Sync + Clone> Extend<&'a T> for List<T> {
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        self.0.extend(iter.into_iter().cloned().map(ListEntry::new));
    }
}

/// A pointer to an element from a [`List`]
///
/// Note that this cannot be used directly to get access to the value in the list. Instead, one
/// must use either one of the methods [`use_w`](Self::use_w) or [`use_rw`](Self::use_rw).
///
/// `ListEntry` implements [`PartialEq`] _AS A POINTER ONLY_. This is so that the properties of a
/// component depend only on which list entry is referenced, and not on the value.
#[allow(clippy::module_name_repetitions)]
pub struct ListEntry<T: 'static + Send + Sync>(ArcMap<Link<T>>);
impl<T: 'static + Send + Sync> PartialEq for ListEntry<T> {
    fn eq(&self, o: &Self) -> bool {
        ArcMap::ptr_eq(&self.0, &o.0)
    }
}
impl<T: 'static + Send + Sync> Clone for ListEntry<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<T: 'static + Send + Sync> ListEntry<T> {
    fn new(t: T) -> Self {
        ListEntry(ArcMap::new(Link::new(t)))
    }
    /// Get a write-only pointer to the element.
    ///
    /// This is generally how an entry is accessed from the component which owns its `List`.
    /// If the entry was passed down from a parent component, then you generally want to call
    /// [`use_w`](Self::use_w) or [`use_rw`](Self::use_rw) instead.
    #[must_use]
    pub fn share(&self) -> Shared<T, super::W> {
        Shared::from_link(self.0.clone())
    }
    /// Get a write pointer to the element as a hook.
    ///
    /// This is the expected way to get write-only access to an entry when it is passed down from a
    /// parent component. If you need to access an entry in the component which owns the list it
    /// belongs to, then you generally need to use [`share`](Self::share) instead.
    #[must_use]
    pub fn use_w<'a, P>(&self, cx: &dioxus_core::Scope<'a, P>) -> &'a mut Shared<T, super::W> {
        let mut opt = Shareable(Some(self.0.clone()));
        Shared::init(cx, &mut opt, || unreachable!(), super::W)
    }
    /// Get a read-write pointer to the element.
    ///
    /// Scope `cx` will be registered as needing update every time the referenced value changes.
    ///
    /// This is the expected ways to get read/write access an entry when it is passed down from a
    /// parent component. If you need to access an entry in the component which owns the list it
    /// belongs to, then you generally need to use [`share`](Self::share) instead.
    #[must_use]
    pub fn use_rw<'a, P>(&self, cx: &dioxus_core::Scope<'a, P>) -> &'a mut Shared<T, super::RW> {
        let mut opt = Shareable(Some(self.0.clone()));
        Shared::init(cx, &mut opt, || unreachable!(), super::RW)
    }
}
