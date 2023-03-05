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
#![deny(clippy::pedantic)]

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
#[derive(Clone, Copy)]
pub struct W;
impl sealed::Flag for W {
    const READ: bool = false;
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

/// Marker trait indicating when one set of actions implies another.
pub trait AsActions<S: r#struct::ShareableStruct, A: r#struct::ActionsFor<S>>:
    r#struct::ActionsFor<S>
{
    fn transmute(s: &Self::WithActions) -> &A::WithActions;
}
impl<S: r#struct::ShareableStruct, A: r#struct::ActionsFor<S>, B: r#struct::ActionsFor<S>>
    AsActions<S, A> for B
where
    Self::WithActions: AsRef<A::WithActions>,
{
    fn transmute(s: &Self::WithActions) -> &A::WithActions {
        s.as_ref()
    }
}
