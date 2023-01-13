//! Module `shared` - Shared values.
//!
//! The easiest way to add a shared value to your dioxus app is to use the
//! [`sharable!`](crate::shareable) macro:
//!
//! NOTE: The type of the shared data must be `Send + Sync`.
//!
//! ```rust
//! # use dioxus::prelude::*;
//! use dioxus_shareables::shareable;
//!
//! shareable!(Var: usize = 900);
//!
//! #[allow(non_snake_case)]
//! pub fn Reader(cx: Scope) -> Element {
//!     let r = *Var.use_rw(&cx).read(); // this component will update when Var changes.
//!     cx.render(rsx! {
//!         "The number is: {r}"
//!     })
//! }
//!
//! #[allow(non_snake_case)]
//! pub fn Writer(cx: Scope) -> Element {
//!     let w1 = Var.use_w(&cx); // this component writes to Var, but does not get updated when Var
//!                              // changes
//!     let w2 = w1.clone();
//!     cx.render(rsx! {
//!         button {
//!             onclick: move |_| { *w1.write() += 1; },
//!             "+"
//!         }
//!         button {
//!             onclick: move |_| { *w2.write() -= 1; },
//!             "-"
//!         }
//!     })
//! }
//! ```

use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
};
use std::{collections::HashMap, sync::Arc};

type LinkUpdateMap = HashMap<usize, (usize, Arc<dyn Send + Sync + Fn()>)>;
/// The actual shared data.
pub(crate) struct Link<T>(RwLock<(T, LinkUpdateMap)>);
impl<T> Link<T> {
    pub(crate) fn new(t: T) -> Self {
        Self(RwLock::new((t, HashMap::new())))
    }
    pub(crate) fn add_listener<F: FnOnce() -> Arc<dyn Send + Sync + Fn()>>(&self, id: usize, f: F) {
        self.0.write().1.entry(id).or_insert_with(|| (0, f())).0 += 1;
    }
    pub(crate) fn drop_listener(&self, id: usize) {
        let mut p = self.0.write();
        let c = if let Some((c, _)) = p.1.get_mut(&id) {
            *c -= 1;
            *c
        } else {
            1
        };
        if c == 0 {
            p.1.remove(&id);
        }
    }
    pub(crate) fn needs_update(&self) {
        for (_id, (_, u)) in self.0.read().1.iter().filter(|&(_, &(ct, _))| ct > 0) {
            u()
        }
    }
    pub(crate) fn borrow(&self) -> MappedRwLockReadGuard<T> {
        RwLockReadGuard::map(self.0.read(), |(r, _)| r)
    }
    pub(crate) fn borrow_mut(&self) -> MappedRwLockWriteGuard<T> {
        RwLockWriteGuard::map(self.0.write(), |(r, _)| r)
    }
}
#[cfg(feature = "debug")]
impl<T: std::fmt::Debug> std::fmt::Debug for Link<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Ok(me) = (self.0).try_borrow() {
            write!(f, "Link({:?})", me.0)
        } else {
            f.write_str("Link::AlreadyBorrowed")
        }
    }
}

/// The storage type for a shared global.
///
/// This is generally not used directly, but it is the type of a static declared with the
/// [`shareable!`](`crate::shareable`) macro, and can be used to construct more complicated shared
/// types.
pub struct Shareable<T>(pub(crate) Option<Arc<Link<T>>>);
impl<T> Shareable<T> {
    pub const fn new() -> Self {
        Self(None)
    }
}
#[cfg(feature = "debug")]
impl<T: std::fmt::Debug> std::fmt::Debug for Shareable<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(me) = &self.0 {
            write!(f, "Shareable::Initialized({:?})", me)
        } else {
            write!(f, "Shareable::Uninitialized")
        }
    }
}

/// Declare a global variable for use as [`Shared`] hook.
///
/// _Example:_
/// ```
/// # use dioxus::prelude::*;
/// dioxus_shareables::shareable!(#[doc(hidden)] Var: usize = 900); // Declares a type Var which can be used to
///                                                                 // access the global.
///
/// fn component(cx: Scope) -> Element {
///     let rw_hook = Var.use_rw(&cx);
///     let w_hook = Var.use_w(&cx);
///     // ...
///     # rsx! {cx, div {}}
/// }
/// ```
#[macro_export]
macro_rules! shareable {
    ($(#[$meta:meta])*$vis:vis $IDENT:ident: $Ty:ty = $($init:tt)*) => {
        $(#[$meta])*
        #[derive(Clone, Copy)]
        $vis struct $IDENT;
        impl $IDENT {
            /// Obtain a RW pointer to the shared value.
            ///
            /// `cx` will be marked as needing update each time you call `.write()` or
            /// `.set()` on this value.
            pub fn use_rw<'a, P>(self,cx: &$crate::reexported::Scope<'a, P>) -> &'a mut $crate::Shared<$Ty, $crate::RW> {
                $crate::shared::Static::_use_rw(self, cx)
            }
            /// Obtain a write pointer to the shared value.
            ///
            /// Note, this doesn't prevent you from reading the data, but raher indicates the
            /// relationship between your component and the data.
            ///
            /// The promise you are making when you `use_w` is that your component does not
            /// need to know when the value changes; i.e., you might read the value, but it
            /// doesn't change what you display.
            pub fn use_w<'a, P>(self,cx: &$crate::reexported::Scope<'a, P>) -> &'a mut $crate::Shared<$Ty, $crate::W> {
                $crate::shared::Static::_use_w(self, cx)
            }
            /// Get a pointer to the value, but don't call 'use_hook'.
            ///
            /// This is generally to be avoided in components, but should be used when the shared
            /// value must be initialized within a loop, or within the initializer of another hook.
            ///
            /// If you don't know why you should be using it, use either [`use_rw`](Self::use_rw)
            /// or [`use_w`](Self::use_w) instead.
            pub fn share(self) -> $crate::Shared<$Ty, $crate::W> {
                $crate::shared::Static::_share(self)
            }
        }
        const _: () = {
            // We declare the static as mutable because we are not thread-safe yet.
            #[allow(non_upper_case_globals)]
            static $IDENT: $crate::reexported::Mutex<$crate::shared::Shareable<$Ty>> = $crate::reexported::Mutex::new($crate::shared::Shareable::new());
            #[doc(hidden)]
            impl $crate::shared::Static for $IDENT {
                type Type = $Ty;
                fn _share(self) -> $crate::Shared<$Ty, $crate::W> {
                    $crate::Shared::from_shareable(&mut $IDENT.lock(), || {$($init)*})
                }
                fn _use_rw<'a, P>(self,cx: &$crate::reexported::Scope<'a, P>) -> &'a mut $crate::Shared<$Ty, $crate::RW> {
                    $crate::Shared::init(cx, &mut $IDENT.lock(), || {$($init)*}, $crate::RW)
                }
                fn _use_w<'a, P>(self,cx: &$crate::reexported::Scope<'a, P>) -> &'a mut $crate::Shared<$Ty, $crate::W> {
                    $crate::Shared::init(cx, &mut $IDENT.lock(), || {$($init)*}, $crate::W)
                }
            }
        };
    };
}

#[doc(hidden)]
pub trait Static {
    type Type;
    fn _share(self) -> Shared<Self::Type, super::W>;
    fn _use_rw<'a, P>(
        self,
        cx: &dioxus_core::Scope<'a, P>,
    ) -> &'a mut Shared<Self::Type, super::RW>;
    fn _use_w<'a, P>(self, cx: &dioxus_core::Scope<'a, P>) -> &'a mut Shared<Self::Type, super::W>;
}

/// A hook to a shared_value.
///
/// This is generally created by calling `use_rw` or `use_w` on a [`shareable!`], or by
/// calling [`ListEntry::use_rw`](`crate::list::ListEntry::use_rw`) or
/// [`ListEntry::use_w`](`crate::list::ListEntry::use_w`).
pub struct Shared<T: 'static, B: 'static> {
    pub(crate) link: Arc<Link<T>>,
    pub id: Option<usize>,
    __: std::marker::PhantomData<B>,
}
impl<T: 'static, B: 'static> Clone for Shared<T, B> {
    fn clone(&self) -> Self {
        if let Some(id) = self.id {
            self.link.add_listener(id, || Arc::new(|| {}))
        }
        Self {
            link: self.link.clone(),
            id: self.id,
            __: std::marker::PhantomData,
        }
    }
}
#[cfg(feature = "dioxus-git")]
#[doc(hidden)]
#[macro_export]
macro_rules! _use_hook { ($cx:expr,$($x:tt)*) => { $cx.use_hook(|| {$($x)*}) } }
#[cfg(not(feature = "dioxus-git"))]
#[doc(hidden)]
#[macro_export]
macro_rules! _use_hook { ($cx:expr,$($x:tt)*) => { $cx.use_hook(|_| {$($x)*}) } }

impl<T: 'static, B: 'static + super::Flag> Shared<T, B> {
    /// Initialize the hook in scope `cx`.
    ///
    /// The shared value will be initialized with `f()` if it hasn't been created yet.
    ///
    /// NOTE: this method should generally not be used directly; instead, shared values are usually
    /// created with [`shareable!`].
    pub fn init<'a, P, F: FnOnce() -> T>(
        cx: &dioxus_core::Scope<'a, P>,
        opt: &mut Shareable<T>,
        f: F,
        _: B,
    ) -> &'a mut Self {
        let id = cx.scope_id().0;
        _use_hook! {cx,
            let mut r: Shared<T, super::W> = Shared::from_shareable(opt, f);
            if B::READ {
                r.id = Some(id);
                r.link.add_listener(id, || cx.schedule_update());
            }
            // SAFETY: Transmuting between Shared<T, A> and Shared<T, B> is safe
            // because the layout of Shared<T, F> does not depend on F.
            unsafe { std::mem::transmute::<_, Self>(r) }
        }
    }
    /// Obtain a write pointer to the shared value and register the change.
    ///
    /// This will mark all components which hold a RW link to the value as needing update.
    pub fn write(&self) -> MappedRwLockWriteGuard<T> {
        self.link.needs_update();
        self.link.borrow_mut()
    }
    /// Obtain a write pointer to the shared value but do not register the change.
    ///
    /// This will not notify consumers of the change to the value.
    pub fn write_silent(&self) -> MappedRwLockWriteGuard<T> {
        self.link.borrow_mut()
    }
    /// Mark the components which hold a RW link to the value as needing update.
    pub fn needs_update(&self) {
        self.link.needs_update();
    }
    /// Set the shared value.
    ///
    /// This marks compoments which hold a RW link to the value as needing update if and only if
    /// the value has changed.
    pub fn set(&self, t: T)
    where
        T: PartialEq,
    {
        if *self.link.borrow() != t {
            *self.write() = t;
        }
    }
    /// Set the shared value to `f(&x)` where `x` is the current value.
    ///
    /// This marks components which hold a RW link to the value as needing update if and only if
    /// the value has changed.
    pub fn set_with<F: Fn(&T) -> T>(&self, f: F)
    where
        T: PartialEq,
    {
        let prev = self.link.borrow();
        let updated = f(&prev);
        if *prev != updated {
            drop(prev);
            *self.write() = updated;
        }
    }
    /// Get the value of the shared data.
    pub fn read(&self) -> MappedRwLockReadGuard<T> {
        self.link.borrow()
    }
    pub fn listeners(&self) -> String {
        format!(
            "{:?}",
            self.link
                .0
                .read()
                .1
                .iter()
                .map(|(&i, &(j, _))| (i, j))
                .collect::<Vec<_>>()
        )
    }
}

impl<T: 'static> Shared<T, super::W> {
    pub(crate) fn from_link(link: Arc<Link<T>>) -> Self {
        Self {
            link,
            id: None,
            __: std::marker::PhantomData,
        }
    }
    #[doc(hidden)]
    pub fn from_shareable<F: FnOnce() -> T>(opt: &mut Shareable<T>, f: F) -> Self {
        if let Some(p) = opt.0.as_ref() {
            Shared {
                link: p.clone(),
                id: None,
                __: std::marker::PhantomData,
            }
        } else {
            let r = Shared {
                link: Arc::new(Link::new(f())),
                id: None,
                __: std::marker::PhantomData,
            };
            opt.0 = Some(r.link.clone());
            r
        }
    }
}

impl<T: 'static, B: 'static> Drop for Shared<T, B> {
    fn drop(&mut self) {
        if let Some(id) = self.id {
            self.link.drop_listener(id);
        }
    }
}
