use std::ptr::NonNull;

/// An wrapper around `std::sync::Arc` which separates the reference counting from the data pointer
/// so that the pointer can be mapped to a subfield and the outer type can be erased.
// Note: `ArcMap` can only be constructed from a valid `std::sync::Arc` (this is how `ArcMap::new`
// works) or by `ArcMap::map`. Both constructions guarantee that `self.inner` will always be a
// vaild pointer during the lifetime of the `ArcMap`.
pub struct ArcMap<T> {
    inner: NonNull<T>,
    outer: Box<dyn Arc>,
}
impl<T> ArcMap<T> {
    pub fn new(t: T) -> Self
    where
        T: 'static + Send + Sync,
    {
        ArcMap::from(std::sync::Arc::new(t))
    }
    pub fn map<U>(self, f: fn(&T) -> &U) -> ArcMap<U> {
        ArcMap {
            // SAFETY:
            //   * self.inner is always vaild if self.outer has not been dropped yet, so it is safe
            //   to dereference here.
            //   * since we have f: Fn<'a>(&'a T) -> &'a U it follows that the lifetime of the
            //   original self.inner bounds the lifetime of the new self.inner, which upholds our
            //   guarantee that the value pointed to will be valid during the lifetime of the
            //   ArcMap.
            inner: f(unsafe { self.inner.as_ref() }).into(),
            outer: self.outer,
        }
    }
    pub fn ptr_eq(a: &Self, b: &Self) -> bool {
        a.inner == b.inner
    }
}
impl<T: 'static + Send + Sync> From<std::sync::Arc<T>> for ArcMap<T> {
    fn from(outer: std::sync::Arc<T>) -> Self {
        ArcMap {
            // SAFETY:
            //  * we never call as_ptr on this.
            //  * This pointer is valid dring the lifetime of outer, and since no method on the
            //  ArcMap modifies outer, this means it is valid during the lifetime of the
            //  ArcMap.
            inner: NonNull::from(&*outer),
            outer: Box::new(outer),
        }
    }
}
impl<T> std::ops::Deref for ArcMap<T> {
    type Target = T;
    fn deref(&self) -> &T {
        // SAFETY: self.inner is always valid if self.outer has not been dropped yet.
        unsafe { self.inner.as_ref() }
    }
}
impl<T> AsRef<T> for ArcMap<T> {
    fn as_ref(&self) -> &T {
        // SAFETY: self.inner is always valid if self.outer has not been dropped yet.
        unsafe { self.inner.as_ref() }
    }
}
impl<T> Clone for ArcMap<T> {
    fn clone(&self) -> Self {
        // SAFETY: The guarantee on the lifetime of self.inner is protected by the guarantee that
        // the std::sync::Arc in self has a positive reference count. Cloning it guarantees that
        // this remains true for the lifetime of this ArcMap as well.
        Self {
            inner: self.inner,
            outer: self.outer.box_clone(),
        }
    }
}
// SAFETY:
//   * since self.outer is actually of Arc<U> for some U and U is Send+Sync by construction (aside
//   from clone(), all methods which construct an ArcMap have a Send+Sync bound), it follows that
//   outer is Send + Sync.
//   * since self.inner is either the same pointer self.outer holds (if constructed with Self::new
//   or with Arc::into).
//   or derived from that pointer by applying some fn(&U)->&T to it, then we can assume that it is
//   either a subfield (which should then be Sync + Send) or a value derived from some 'static
//   (which is therefore necessarily Send+Sync as well).
unsafe impl<T: Sync + Send> Send for ArcMap<T> {}
// SAFETY: (see above)
unsafe impl<T: Sync + Send> Sync for ArcMap<T> {}

trait Arc: Send + Sync {
    fn box_clone(&self) -> Box<dyn Arc>;
}
impl<T: 'static + Send + Sync> Arc for std::sync::Arc<T> {
    fn box_clone(&self) -> Box<dyn Arc> {
        Box::new(self.clone())
    }
}
