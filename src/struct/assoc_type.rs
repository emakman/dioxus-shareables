//! Macros for inherent associated types.

/// Hack to get stable inherent associated associated types.
///
/// If you are getting weird errors about `AssocSubstruct`, `AssocField` constraints not being
/// correct, then this is probably the culprit. Spellcheck uses of `struct_assoc_type!` and
/// `struct_actions!`.
#[macro_export]
#[allow(clippy::module_name_repetitions)]
macro_rules! struct_assoc_type {
    ({$Struct:ty}::Actions::$action:ident) => {
        $crate::reexported::paste! {
            $crate::struct_assoc_type! {
                @[AssocAction][$Struct](<<$Struct as $crate::r#struct::ShareableStruct>::ActionData as )([<Action $action:camel>])(>::Type)
            }
        }
    };
    ($Struct:ident::Actions::$action:ident) => {
        $crate::reexported::paste! {
            $crate::struct_assoc_type! {
                @[AssocAction][$Struct](<<$Struct as $crate::r#struct::ShareableStruct>::ActionData as )([<Action $action:camel>])(>::Type)
            }
        }
    };
    ({$Struct:ty}::Fields::$field:ident) => {
        $crate::reexported::paste! {
            $crate::struct_assoc_type! {
                @[AssocField][$Struct](<<$Struct as $crate::r#struct::ShareableStruct>::FieldData as )([<Field $field:camel>])(>::Type)
            }
        }
    };
    ($Struct:ident::Fields::$field:ident) => {
        $crate::reexported::paste! {
            $crate::struct_assoc_type! {
                @[AssocField][$Struct](<<$Struct as $crate::r#struct::ShareableStruct>::FieldData as )([<Field $field:camel>])(>::Type)
            }
        }
    };
    ({$Struct:ty}::Substructs::$field:ident) => {
        $crate::reexported::paste! {
            $crate::struct_assoc_type! {
                @[AssocSubstruct][$Struct](<<$Struct as $crate::r#struct::ShareableStruct>::FieldData as )([<Field $field:camel>])(>::Type)
            }
        }
    };
    ($Struct:ident::Substructs::$field:ident) => {
        $crate::reexported::paste! {
            $crate::struct_assoc_type! {
                @[AssocSubstruct][$Struct](<<$Struct as $crate::r#struct::ShareableStruct>::FieldData as )([<Field $field:camel>])(>::Type)
            }
        }
    };
    ($Struct:ident::Shareable) => { $crate::arcmap::ArcMap<<$Struct as $crate::r#struct::ShareableStruct>::Content> };
    (impl $Struct:ident::Actions::$action:ident for $T:ty = $($what:tt)*) => {
        $crate::reexported::paste! {
            $crate::struct_assoc_type! {
                @[AssocAction][$Struct](impl)([<Action $action:camel>])(for $T { type Type = $($what)*; })
            }
        }
    };
    (impl $Struct:ident::Fields::$field:ident for $T:ty = $($what:tt)*) => {
        $crate::reexported::paste! {
            $crate::struct_assoc_type! {
                @[AssocField][$Struct](impl)([<Field $field:camel>])( for $T { type Type = $($what)*; })
            }
        }
    };
    (impl $Struct:ident::Substructs::$field:ident for $T:ty = $($what:tt)*) => {
        $crate::reexported::paste! {
            $crate::struct_assoc_type! {
                @[AssocSubstruct][$Struct](impl)([<Field $field:camel>])( for $T { type Type = $($what)*; })
            }
        }
    };
    ($(@[$AssocType:ident][$Struct:ty]($($before:tt)*)($($x:tt)*)($($after:tt)*))*) => {
        $($($before)*$crate::r#struct::assoc_type::$AssocType<
            $Struct,
            {$crate::r#struct::assoc_type::seg_str(stringify!{$($x)*}, 0)},
            {$crate::r#struct::assoc_type::seg_str(stringify!{$($x)*}, 1)},
            {$crate::r#struct::assoc_type::seg_str(stringify!{$($x)*}, 2)},
            {$crate::r#struct::assoc_type::seg_str(stringify!{$($x)*}, 3)},
            {$crate::r#struct::assoc_type::seg_str(stringify!{$($x)*}, 4)},
            {$crate::r#struct::assoc_type::seg_str(stringify!{$($x)*}, 5)},
            {$crate::r#struct::assoc_type::seg_str(stringify!{$($x)*}, 6)},
            {$crate::r#struct::assoc_type::seg_str(stringify!{$($x)*}, 7)},
            {$crate::r#struct::assoc_type::seg_str(stringify!{$($x)*}, 8)},
            {$crate::r#struct::assoc_type::seg_str(stringify!{$($x)*}, 9)},
            {$crate::r#struct::assoc_type::seg_str(stringify!{$($x)*}, 10)},
            {$crate::r#struct::assoc_type::seg_str(stringify!{$($x)*}, 11)},
            {$crate::r#struct::assoc_type::seg_str(stringify!{$($x)*}, 12)},
            {$crate::r#struct::assoc_type::seg_str(stringify!{$($x)*}, 13)},
            {$crate::r#struct::assoc_type::seg_str(stringify!{$($x)*}, 14)},
            {$crate::r#struct::assoc_type::seg_str(stringify!{$($x)*}, 15)},
        >$($after)*)*
    }
}

/// &'static str is not allowed for const generics, but we can imitate a &'static [u8; 256] bound
/// using a lot of ints here.
pub trait AssocSubstruct<
    S: super::ShareableStruct,
    const _0: u128,
    const _1: u128,
    const _2: u128,
    const _3: u128,
    const _4: u128,
    const _5: u128,
    const _6: u128,
    const _7: u128,
    const _8: u128,
    const _9: u128,
    const _10: u128,
    const _11: u128,
    const _12: u128,
    const _13: u128,
    const _14: u128,
    const _15: u128,
>
{
    type Type: super::Substruct<Of = S>;
}

/// &'static str is not allowed for const generics, but we can imitate a &'static [u8; 256] bound
/// using a lot of ints here.
pub trait AssocField<
    S: super::ShareableStruct,
    const _0: u128,
    const _1: u128,
    const _2: u128,
    const _3: u128,
    const _4: u128,
    const _5: u128,
    const _6: u128,
    const _7: u128,
    const _8: u128,
    const _9: u128,
    const _10: u128,
    const _11: u128,
    const _12: u128,
    const _13: u128,
    const _14: u128,
    const _15: u128,
>
{
    type Type: super::Field<Of = S>;
}

/// &'static str is not allowed for const generics, but we can imitate a &'static [u8; 256] bound
/// using a lot of ints here.
pub trait AssocAction<
    S: super::ShareableStruct,
    const _0: u128,
    const _1: u128,
    const _2: u128,
    const _3: u128,
    const _4: u128,
    const _5: u128,
    const _6: u128,
    const _7: u128,
    const _8: u128,
    const _9: u128,
    const _10: u128,
    const _11: u128,
    const _12: u128,
    const _13: u128,
    const _14: u128,
    const _15: u128,
>
{
    type Type: super::ActionsFor<S>;
}
#[must_use]
pub const fn seg_str(s: &'static str, r: usize) -> u128 {
    let mut i = 0usize;
    let mut c = 0;
    loop {
        if i >= 16 || r + i >= s.len() {
            return c;
        }
        c += (s.as_bytes()[r + i] as u128) << (8 * i);
        i += 1;
    }
}
