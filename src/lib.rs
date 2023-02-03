//! Crate [`dioxus-shareables`](crate)
//!
//! This crate provides abstractions for global resource sharing in
//! [`dioxus`](https://docs.rs/dioxus) similar to `fermi`, but with a slightly different data
//! model, and some extensions for shared structures.
//!
//! The primary interfaces for the crate are [`Shared`], [`shareable_struct`] and [`List`]
//!
//! `dioxus` is still under development; if you're using the latest nightly version of `dioxus`
//! then your `Cargo.toml` should look something like this:
//! ```Cargo.toml
//! ...
//! [dependencies]
//! ...
//! dioxus-shareables = { version = "0.3.0", features = ["dixous-git"] }
//! ...
//! [replace]
//! "dioxus-core:0.3.0" = { git = 'https://github.com/dioxuslabs/dioxus' }
//! "dioxus-hooks:0.3.0" = { git = 'https://github.com/dioxuslabs/dioxus' }
//!
//! ```

pub mod shared;
pub use shared::Shared;

pub mod list;
pub use list::{List, ListEntry};

pub mod arcmap;

#[doc(hidden)]
pub mod r#struct;

#[doc(hidden)]
pub mod reexported {
    pub use dioxus_core::Scope;
    pub use paste::paste;
}

#[doc(hidden)]
mod sealed {
    pub trait Flag {
        const READ: bool;
    }

    pub trait InductiveMarkerTuple {
        type __Base;
        type __Step;
        type __Decons;
        fn __base(&self) -> Self::__Base;
        fn __step(&self) -> Self::__Step;
    }
    macro_rules! impl_InductiveMarkerTuple_for {
        () => {};
        ($T:ident$($(,$U:ident$({$ct:tt})?)+)?) => {
            impl<$T: Copy$($(,$U: Copy)+)?> InductiveMarkerTuple for ($($($U,)+)?$T,) {
                type __Base = ($($($U,)+)?);
                type __Step = $T;
                type __Decons = super::Decons<Self::__Base, Self::__Step>;
                fn __base(&self) -> Self::__Base {
                    paste::paste! {
                        let ($($([<__ $U:lower>],)+)?_,) = *self;
                        $(($([<__ $U:lower>],)+))?
                    }
                }
                fn __step(&self) -> Self::__Step {
                    let ($($(_ $($ct)?,)+)?r,) = *self;
                    r
                }
            }
            impl_InductiveMarkerTuple_for! ($($($U),+)?);
        }
    }
    impl_InductiveMarkerTuple_for!(
        AA, AB, AC, AD, AE, AF, AG, AH, AI, AJ, AK, AL, AM, AN, AO, AP, AQ, AR, AS, AT, AU, AV, AW,
        AX, AY, AZ, BA, BB, BC, BD, BE, BF, BG, BH, BI, BJ, BK, BL, BM, BN, BO, BP, BQ, BR, BS, BT,
        BU, BV, BW, BX, BY, BZ
    );

    pub trait InitType: Sized + Copy {
        fn __init_field<P, T: 'static + Send + Sync, S: crate::shared::Static<Type = T>>(
            cx: &dioxus_core::Scope<P>,
            _: &mut Option<crate::shared::Shared<T, Self>>,
            _: S,
        );
        fn __share_field<T: 'static + Send + Sync, S: crate::shared::Static<Type = T>>(
            _: &mut Option<crate::Shared<T, Self>>,
            _: S,
        );
    }
    impl InitType for () {
        fn __init_field<P, T: 'static + Send + Sync, S: crate::shared::Static<Type = T>>(
            _: &dioxus_core::Scope<P>,
            _: &mut Option<crate::Shared<T, Self>>,
            _: S,
        ) {
        }
        fn __share_field<T: 'static + Send + Sync, S: crate::shared::Static<Type = T>>(
            _: &mut Option<crate::Shared<T, Self>>,
            _: S,
        ) {
        }
    }
}
/// A type flag for shared pointers.
///
/// This trait is implemented for [`W`] and [`RW`], the marker types which indicate the behavior of
/// a [`Shared`] hook.
pub trait Flag: sealed::Flag {}
impl<T: sealed::Flag> Flag for T {}

/// A type flag for fields in shared structures.
///
/// This trait is implemented for [`W`], [`RW`], and [`()`], so it is either a Flag or the unit
/// type (which is used to indicate a field is not initialized.)
pub trait InitType: sealed::InitType {
    #[doc(hidden)]
    fn init_field<P, T: 'static + Send + Sync, S: shared::Static<Type = T>>(
        cx: &dioxus_core::Scope<P>,
        f: &mut Option<Shared<T, Self>>,
        s: S,
    ) {
        Self::__init_field(cx, f, s)
    }
    #[doc(hidden)]
    fn share_field<T: 'static + Send + Sync, S: crate::shared::Static<Type = T>>(
        f: &mut Option<crate::Shared<T, Self>>,
        s: S,
    ) {
        Self::__share_field(f, s)
    }
}
impl<T: sealed::InitType> InitType for T {}

/// Marker for an access to shared data which is used for writing but not reading.
///
/// The primary promise for such an access is that it does not effect component display.
#[derive(Clone, Copy)]
pub struct W;
impl sealed::Flag for W {
    const READ: bool = false;
}
impl sealed::InitType for W {
    fn __init_field<P, T: 'static + Send + Sync, S: shared::Static<Type = T>>(
        _: &dioxus_core::Scope<P>,
        f: &mut Option<Shared<T, Self>>,
        s: S,
    ) {
        if f.is_none() {
            *f = Some(s._share());
        }
    }
    fn __share_field<T: 'static + Send + Sync, S: crate::shared::Static<Type = T>>(
        f: &mut Option<crate::Shared<T, Self>>,
        s: S,
    ) {
        if f.is_none() {
            *f = Some(s._share());
        }
    }
}

/// Marker for an access to shared data which is used for reading.
///
/// Components which hold a `RW` handle are marked as needing update whenever that handle is
/// written to.
#[derive(Clone, Copy)]
pub struct RW;
impl sealed::Flag for RW {
    const READ: bool = true;
}
impl sealed::InitType for RW {
    fn __init_field<P, T: 'static + Send + Sync, S: shared::Static<Type = T>>(
        cx: &dioxus_core::Scope<P>,
        f: &mut Option<Shared<T, Self>>,
        s: S,
    ) {
        if f.is_none() {
            let id = cx.scope_id().0;
            let mut r = s._share();
            r.id = Some(id);
            r.link.add_listener(id, || cx.schedule_update());
            // SAFETY:
            //   * Shared<T, W> and Shared<T, RW> are layed out identically in memory.
            *f = Some(unsafe { std::mem::transmute(r) });
        }
    }
    fn __share_field<T: 'static + Send + Sync, S: crate::shared::Static<Type = T>>(
        _: &mut Option<crate::Shared<T, Self>>,
        _: S,
    ) {
        unreachable!()
    }
}

#[doc(hidden)]
pub trait InductiveMarkerTuple: sealed::InductiveMarkerTuple {
    type Base;
    type Step;
    type Decons;
    fn base(&self) -> Self::Base;
    fn step(&self) -> Self::Step;
}
impl<T: sealed::InductiveMarkerTuple> InductiveMarkerTuple for T {
    type Base = T::__Base;
    type Step = T::__Step;
    type Decons = T::__Decons;
    fn base(&self) -> Self::Base {
        self.__base()
    }
    fn step(&self) -> Self::Step {
        self.__step()
    }
}
#[doc(hidden)]
pub struct Decons<T, U>(T, U);
