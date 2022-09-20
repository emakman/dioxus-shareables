//! Module `shared` - Shared values.
//!
//! The easiest way to add a shared value to your dioxus app is to use the
//! [`sharable!`](crate::shareable) macro:
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

use std::{
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    sync::{Arc, Weak},
};

/// The actual shared data.
pub(crate) struct Link<T>(RefCell<(T, HashMap<usize, (usize, Arc<dyn Fn()>)>)>);
impl<T> Link<T> {
    pub(crate) fn new(t: T) -> Self {
        Self(RefCell::new((t, HashMap::new())))
    }
    pub(crate) fn add_listener<F: FnOnce() -> Arc<dyn Fn()>>(&self, id: usize, f: F) {
        #[cfg(feature = "debugging")]
        unsafe {
            super::LOG(id, "INCREASING LISTENER COUNT")
        };
        self.0
            .borrow_mut()
            .1
            .entry(id)
            .or_insert_with(|| {
                #[cfg(feature = "debugging")]
                unsafe {
                    super::LOG(id, "ADDING NEW LISTENER")
                };
                (0, f())
            })
            .0 += 1;
    }
    pub(crate) fn drop_listener(&self, id: usize) {
        #[cfg(feature = "debugging")]
        unsafe {
            super::LOG(id, "DECREASING LISTENER COUNT")
        };
        let mut p = self.0.borrow_mut();
        let c = if let Some((c, _)) = p.1.get_mut(&id) {
            *c -= 1;
            *c
        } else {
            1
        };
        if c == 0 {
            #[cfg(feature = "debugging")]
            unsafe {
                super::LOG(id, "DROPPING LISTENER")
            };
            p.1.remove(&id);
        }
    }
    pub(crate) fn needs_update(&self) {
        for (_id, (_, u)) in self.0.borrow().1.iter().filter(|&(_, &(ct, _))| ct > 0) {
            #[cfg(feature = "debugging")]
            unsafe {
                super::LOG(*_id, "MARKING FOR UPDATE")
            };
            u()
        }
    }
    pub(crate) fn borrow(&self) -> Ref<T> {
        Ref::map(self.0.borrow(), |(r, _)| r)
    }
    pub(crate) fn borrow_mut(&self) -> RefMut<T> {
        RefMut::map(self.0.borrow_mut(), |(r, _)| r)
    }
}

/// The storage type for a shared global.
///
/// This is generally not used directly, but it is the type of a static declared with the
/// [`shareable!`](`crate::shareable`) macro, and can be used to construct more complicated shared
/// types.
pub struct Shareable<T>(pub(crate) Option<Weak<Link<T>>>);
impl<T> Shareable<T> {
    pub const fn new() -> Self {
        Self(None)
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
            /// must be initialized within a loop, or within the initializer of another hook.
            ///
            /// If you don't know why you should be using it, use either [`use_rw`](Self::use_rw)
            /// or [`use_w`](Self::use_w) instead.
            pub fn share(self) -> $crate::Shared<$Ty, $crate::W> {
                $crate::shared::Static::_share(self)
            }
        }
        const _: () = {
            #[allow(non_upper_case_globals)]
            static mut $IDENT: $crate::shared::Shareable<$Ty> = $crate::shared::Shareable::new();
            #[doc(hidden)]
            impl $crate::shared::Static for $IDENT {
                type Type = $Ty;
                fn _share(self) -> $crate::Shared<$Ty, $crate::W> {
                    $crate::Shared::from_shareable(unsafe { &mut $IDENT }, || {$($init)*})
                }
                fn _use_rw<'a, P>(self,cx: &$crate::reexported::Scope<'a, P>) -> &'a mut $crate::Shared<$Ty, $crate::RW> {
                    $crate::Shared::init(cx, unsafe { &mut $IDENT }, || {$($init)*}, $crate::RW)
                }
                fn _use_w<'a, P>(self,cx: &$crate::reexported::Scope<'a, P>) -> &'a mut $crate::Shared<$Ty, $crate::W> {
                    $crate::Shared::init(cx, unsafe { &mut $IDENT }, || {$($init)*}, $crate::W)
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
        cx.use_hook(|_| {
            let mut r: Shared<T, super::W> = Shared::from_shareable(opt, f);
            if B::READ {
                r.id = Some(id);
                r.link.add_listener(id, || cx.schedule_update());
            }
            unsafe { std::mem::transmute::<_, Self>(r) }
        })
    }
    /// Obtain a write pointer to the shared value and register the change.
    ///
    /// This will mark all components which hold a RW link to the value as needing update.
    pub fn write(&self) -> RefMut<T> {
        self.link.needs_update();
        self.link.borrow_mut()
    }
    /// Obtain a write pointer to the shared value but do not register the change.
    ///
    /// This will not notify consumers of the change to the value.
    pub fn write_silent(&self) -> RefMut<T> {
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
    pub fn read(&self) -> Ref<T> {
        self.link.borrow()
    }
    pub fn listeners<'a>(&'a self) -> String {
        format!(
            "{:?}",
            self.link
                .0
                .borrow()
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
        if let Some(Some(p)) = opt.0.as_ref().map(Weak::upgrade) {
            Shared {
                link: p,
                id: None,
                __: std::marker::PhantomData,
            }
        } else {
            let r = Shared {
                link: Arc::new(Link::new(f())),
                id: None,
                __: std::marker::PhantomData,
            };
            opt.0 = Some(Arc::downgrade(&r.link));
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
