//! Crate [`dioxus-shareables`](crate)
//!
//! This crate provides abstractions for global resource sharing in
//! [`dioxus`](https://docs.rs/dioxus) similar to `fermi`, but with a slightly different data
//! model, and some extensions for shared structures.
pub mod shared;
pub use shared::Shared;

pub mod list;
pub use list::{List, ListEntry};

#[doc(hidden)]
pub mod r#struct;

#[doc(hidden)]
pub mod reexported {
    pub use dioxus_core::Scope;
    pub use paste::paste;
}

#[cfg(feature = "debugging")]
pub static mut LOG: fn(usize, &str) = |_component, _msg| {};

#[doc(hidden)]
mod sealed {
    pub trait Flag {
        const READ: bool;
    }
    pub trait ImpliesFlag<F: Flag>: Flag {}
    impl<F: Flag> ImpliesFlag<F> for F {}

    pub trait InductiveTuple {
        type __Base;
        type __Step;
        type __Decons;
    }
    macro_rules! impl_InductiveTuple_for {
        () => {};
        ($T:ident$(,$U:ident)*) => {
            impl<$T$(,$U)*> InductiveTuple for ($($U,)*$T,) {
                type __Base = ($($U,)*);
                type __Step = $T;
                type __Decons = super::Decons<Self::__Base, Self::__Step>;
            }
            impl_InductiveTuple_for! ($($U),*);
        }
    }
    impl_InductiveTuple_for!(
        AA, AB, AC, AD, AE, AF, AG, AH, AI, AJ, AK, AL, AM, AN, AO, AP, AQ, AR, AS, AT, AU, AV, AW,
        AX, AY, AZ, BA, BB, BC, BD, BE, BF, BG, BH, BI, BJ, BK, BL, BM, BN, BO, BP, BQ, BR, BS, BT,
        BU, BV, BW, BX, BY, BZ
    );
}
/// A type flag for shared pointers.
///
/// This trait is implemented for [`W`] and [`RW`], the marker types which indicate the behavior of
/// a [`Shared`] hook.
pub trait Flag: sealed::Flag {}
impl<T: sealed::Flag> Flag for T {}

/// Marker for an access to shared data which is used for writing but not reading.
///
/// The primary promise for such an access is that it does not effect component display.
pub struct W;
impl sealed::Flag for W {
    const READ: bool = false;
}

/// Marker for an access to shared data which is used for reading.
///
/// Components which hold a `RW` handle are marked as needing update whenever that handle is
/// written to.
pub struct RW;
impl sealed::Flag for RW {
    const READ: bool = true;
}

#[doc(hidden)]
pub trait InductiveTuple: sealed::InductiveTuple {
    type Base;
    type Step;
    type Decons;
}
impl<T: sealed::InductiveTuple> InductiveTuple for T {
    type Base = T::__Base;
    type Step = T::__Step;
    type Decons = T::__Decons;
}
#[doc(hidden)]
pub struct Decons<T, U>(T, U);
