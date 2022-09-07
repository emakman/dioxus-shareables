/// Create a `struct` definition for a global.
///
/// The idea is that each field of the struct will be stored in a separate global, and loaded only
/// when requested. The actions block describes possible ways of using the struct in terms of what
/// type of access ([`W`](crate::W) or [`RW`](crate::RW)) they need to fields of the struct.
///
/// The struct can then be initialized using an "action" which describes which fields we need which
/// type of access to.
///
/// ```
///        # use dioxus::prelude::*;
///        dioxus_shareables::shareable_struct! {
///            pub struct Fuzzy {
///                wuzzy: u8 = 17,
///                was_a: u16 = 59,
///                was_he: &'static str = "bear?",
///            }
///            actions for Puzzle {
///               pub WAS: pub WasTrait = W[was_a, was_he]; // declares an WAS constant, as well an
///                                                         // equivalent trait.
///               INIT = W[wuzzy, was_a], RW[was_he]; // declares the INIT constant, but no
///                                                   // equivalent trait.
///            }
///        };
///        impl<A: FuzzyActions> Fuzzy<A> {
///            pub fn method(&self) where A: WasTrait {
///               let me = self.with_actions(WAS); // Pending updates to the rust trait system, we
///                                                // have to typecast here to get a Fuzzy<WAS>.
///               *me.was_he().write() = "bare!"; // We have write access to was_he
///               // self.wuzzy(); // but this would fail because we don't have access to wuzzy.
///               // ...
///            }
///        }
///        // ...
///        fn component(cx: Scope) -> Element {
///             let fuzzy = Fuzzy::use_(&cx, INIT); // This creates the hooks for the struct and initializes it
///                                                 // from the necessary globals.
///             // ...
///             fuzzy.method(); // This is ok, since the INIT action includes everything the WAS action does.
///             // ...
///             # rsx! {cx, div {}}
///        }
/// ```
#[macro_export]
macro_rules! shareable_struct {
    (
        $vis:vis struct $Struct:ident {
            $($field:ident: $T:ty = $init:expr),*$(,)?
        }
        actions for $_:ident {
            $(
                $action_vis:vis $ACTION:ident$(: $trait_vis:vis $Trait:ident)?
                    = $(W[$($w:ident),*])?$(,)?$(RW[$($rw:ident),*])?;
            )*
        }
    ) => {
        $crate::reexported::paste! {
            #[doc(hidden)]
            $vis trait [<$Struct Actions>]: $($crate::r#struct::InitField<[<$Struct Field_ $field>]> + )*'static {
                fn load<P>(cx: &$crate::reexported::Scope<P>, s: &mut $Struct<Self>) {
                    $(<Self as $crate::r#struct::InitField<[<$Struct Field_ $field>]>>::load(cx, &mut s.$field);)*
                }
            }
            impl<T: $($crate::r#struct::InitField<[<$Struct Field_ $field>]> + )*'static> [<$Struct Actions>] for T {}

            #[doc(hidden)]
            $vis trait [<$Struct WriteActions>]: $($crate::r#struct::LoadFieldWrite<[<$Struct Field_ $field>]> + )*[<$Struct Actions>] {
                fn load(s: &mut $Struct<Self>) {
                    $(<Self as $crate::r#struct::LoadFieldWrite<[<$Struct Field_ $field>]>>::load_write(&mut s.$field);)*
                }
            }
            impl<T: $($crate::r#struct::LoadFieldWrite<[<$Struct Field_ $field>]> + )*[<$Struct Actions>]> [<$Struct WriteActions>] for T {}

            $(
                $crate::shareable!(#[doc(hidden)]#[allow(non_camel_case_types)]$vis [<$Struct Field_ $field>]: $T = $init);
            )*
            $crate::__struct!(@$Struct[]$([<$Struct Field_ $field>])*);
            $vis struct $Struct<A: [<$Struct Actions>] + ?Sized> {
                $($field: Option<$crate::Shared<$T, <A as $crate::r#struct::InitField<[<$Struct Field_ $field>]>>::Flag>>,)*
                _marker: std::marker::PhantomData<A>,
            }
            impl<A: [<$Struct Actions>]> $Struct<A> {
                /// Hook to create an pointer to the shared data.
                ///
                /// The second argument describes which fields of the struct are needed.
                /// Generally this should be one of the Actions constants created in this same macro invocation.
                ///
                /// _Generated in macro shared_struct!_
                $vis fn use_<P>(cx: &$crate::reexported::Scope<P>, _: A) -> Self {
                    let mut r = Self {
                        $($field: None,)*
                        _marker: std::marker::PhantomData
                    };
                    <A as [<$Struct Actions>]>::load(cx, &mut r);
                    r
                }
                /// Create a pointer to the shared data without a hook.
                ///
                /// The second argument describes which fields of the struct are needed.
                /// Generally this should be one of the Actions constants created in this same macro invocation.
                ///
                /// It is almost always preferable to use the [`use_`](Self::use_) hook instead, so as to avoid
                /// incrementing/decrementing reference counters each time the component is updated.
                ///
                /// _Generated in macro shared_struct!_
                $vis fn share_without_hook(_: A) -> Self where A: [<$Struct WriteActions>] {
                    let mut r = Self {
                        $($field: None,)*
                        _marker: std::marker::PhantomData
                    };
                    <A as [<$Struct WriteActions>]>::load(&mut r);
                    r
                }
                $(
                    $vis fn $field(&self) -> $crate::Shared<$T, <A as $crate::r#struct::InitField<[<$Struct Field_ $field>]>>::Flag>
                    where
                        A: $crate::r#struct::LoadField<[<$Struct Field_ $field>]>
                    {
                        match &self.$field {
                            Some(r) => r.clone(),
                            _ => unreachable!()
                        }
                    }
                )*
                $vis fn with_actions<B: [<$Struct Actions>]>(&self, _: B) -> &$Struct<B> where A: [<Implies $Struct Actions>]<B> {
                    unsafe { std::mem::transmute(self) }
                }
            }
            #[doc(hidden)]
            $vis trait [<Implies $Struct Actions>]<B: [<$Struct Actions>]>: [<$Struct Actions>] {}
            impl<A: [<$Struct Actions>], B: [<$Struct Actions>]> [<Implies $Struct Actions>]<B> for A where
                $(<A as $crate::r#struct::InitField<[<$Struct Field_ $field>]>>::Flag:
                    $crate::r#struct::Implies<<B as $crate::r#struct::InitField<[<$Struct Field_ $field>]>>::Flag>,)*
            {}
            $(
                $action_vis const $ACTION: [($($($crate::r#struct::LoadW<[<$Struct Field_ $w>]>,)*)?$($($crate::r#struct::LoadRW<[<$Struct Field_ $rw>]>,)*)?); 1]
                    = [($($($crate::r#struct::LoadW([<$Struct Field_ $w>]),)*)?$($($crate::r#struct::LoadRW([<$Struct Field_ $rw>]),)*)?)];
                $crate::__struct! {@@
                    $([$trait_vis $Trait])?
                    [[<Implies $Struct Actions>]<[($($($crate::r#struct::LoadW<[<$Struct Field_ $w>]>,)*)?$($($crate::r#struct::LoadRW<[<$Struct Field_ $rw>]>,)*)?); 1]>]
                }
            )*
        }
    };
}
#[doc(hidden)]
#[macro_export]
macro_rules! __struct {
    (@$Struct:ident$before:tt) => {};
    (@$Struct:ident[$($before:ident)*]$field:ident$($after:ident)*) => {
        $(
            impl<T> $crate::r#struct::InitField<$before> for $crate::Decons<T, $crate::r#struct::LoadW<$field>>
            where
                [T; 1]: $crate::r#struct::InitField<$before>
            {
                type Flag = <[T; 1] as $crate::r#struct::InitField<$before>>::Flag;
                fn load<P>(
                    cx: &$crate::reexported::Scope<P>,
                    o: &mut Option<$crate::Shared<<$before as $crate::shared::Static>::Type, Self::Flag>>,
                ) {
                    <[T; 1] as $crate::r#struct::InitField<$before>>::load(cx, o)
                }
            }
            impl<T> $crate::r#struct::LoadFieldWrite<$before> for $crate::Decons<T, $crate::r#struct::LoadW<$field>>
            where
                [T; 1]: $crate::r#struct::LoadFieldWrite<$before>
            {
                fn load_write(o: &mut Option<$crate::Shared<<$before as $crate::shared::Static>::Type, Self::Flag>>) {
                    <[T; 1] as $crate::r#struct::LoadFieldWrite<$before>>::load_write(o)
                }
            }
            impl<T> $crate::r#struct::InitField<$before> for $crate::Decons<T, $crate::r#struct::LoadRW<$field>>
            where
                [T; 1]: $crate::r#struct::InitField<$before>
            {
                type Flag = <[T; 1] as $crate::r#struct::InitField<$before>>::Flag;
                fn load<P>(
                    cx: &$crate::reexported::Scope<P>,
                    o: &mut Option<$crate::Shared<<$before as $crate::shared::Static>::Type, Self::Flag>>,
                ) {
                    <[T; 1] as $crate::r#struct::InitField<$before>>::load(cx, o)
                }
            }
        )*
        impl<T> $crate::r#struct::InitField<$field> for $crate::Decons<T, $crate::r#struct::LoadW<$field>> {
            type Flag = $crate::W;
            fn load<P>(
                cx: &$crate::reexported::Scope<P>,
                o: &mut Option<$crate::Shared<<$field as $crate::shared::Static>::Type, Self::Flag>>,
            ) {
                *o = Some($field.use_w(cx));
            }
        }
        impl<T> $crate::r#struct::LoadFieldWrite<$field> for $crate::Decons<T, $crate::r#struct::LoadW<$field>> {
            fn load_write(o: &mut Option<$crate::Shared<<$field as $crate::shared::Static>::Type, Self::Flag>>) {
                *o = Some($crate::shared::Static::share_without_hook($field));
            }
        }
        impl<T> $crate::r#struct::InitField<$field> for $crate::Decons<T, $crate::r#struct::LoadRW<$field>> {
            type Flag = $crate::RW;
            fn load<P>(
                cx: &$crate::reexported::Scope<P>,
                o: &mut Option<$crate::Shared<<$field as $crate::shared::Static>::Type, Self::Flag>>,
            ) {
                *o = Some($field.use_rw(cx));
            }
        }
        $(
            impl<T> $crate::r#struct::InitField<$after> for $crate::Decons<T, $crate::r#struct::LoadW<$field>>
            where
                [T; 1]: $crate::r#struct::InitField<$after>
            {
                type Flag = <[T; 1] as $crate::r#struct::InitField<$after>>::Flag;
                fn load<P>(
                    cx: &$crate::reexported::Scope<P>,
                    o: &mut Option<$crate::Shared<<$after as $crate::shared::Static>::Type, Self::Flag>>,
                ) {
                    <[T; 1] as $crate::r#struct::InitField<$after>>::load(cx, o)
                }
            }
            impl<T> $crate::r#struct::LoadFieldWrite<$after> for $crate::Decons<T, $crate::r#struct::LoadW<$field>>
            where
                [T; 1]: $crate::r#struct::LoadFieldWrite<$after>
            {
                fn load_write(o: &mut Option<$crate::Shared<<$after as $crate::shared::Static>::Type, Self::Flag>>) {
                    <[T; 1] as $crate::r#struct::LoadFieldWrite<$after>>::load_write(o)
                }
            }
            impl<T> $crate::r#struct::InitField<$after> for $crate::Decons<T, $crate::r#struct::LoadRW<$field>>
            where
                [T; 1]: $crate::r#struct::InitField<$after>
            {
                type Flag = <[T; 1] as $crate::r#struct::InitField<$after>>::Flag;
                fn load<P>(
                    cx: &$crate::reexported::Scope<P>,
                    o: &mut Option<$crate::Shared<<$after as $crate::shared::Static>::Type, Self::Flag>>,
                ) {
                    <[T; 1] as $crate::r#struct::InitField<$after>>::load(cx, o)
                }
            }
        )*
        $crate::__struct!(@$Struct[$($before)*$field]$($after)*);
    };
    (@@$_:tt) => {};
    (@@[$vis:vis $Trait:ident][$($Equiv:tt)*]) => {
        $vis trait $Trait: $($Equiv)* {}
        impl<T: $($Equiv)*> $Trait for T {}
    };
}

pub trait InitField<F: crate::shared::Static> {
    type Flag;
    fn load<P>(_: &dioxus_core::Scope<P>, _: &mut Option<crate::Shared<F::Type, Self::Flag>>);
}
pub trait LoadField<F: crate::shared::Static>: InitField<F> {}
impl<F: crate::shared::Static, T: InitField<F>> LoadField<F> for T where T::Flag: super::Flag {}

pub trait LoadFieldWrite<F: crate::shared::Static>: InitField<F> {
    fn load_write(_: &mut Option<crate::Shared<F::Type, Self::Flag>>);
}

impl<F: crate::shared::Static> InitField<F> for [(); 1] {
    type Flag = ();
    fn load<P>(_: &dioxus_core::Scope<P>, _: &mut Option<crate::Shared<F::Type, ()>>) {}
}
impl<F: crate::shared::Static> LoadFieldWrite<F> for [(); 1] {
    fn load_write(_: &mut Option<crate::Shared<F::Type, ()>>) {}
}

impl<F: crate::shared::Static, T: crate::InductiveTuple> InitField<F> for [T; 1]
where
    T::Decons: InitField<F>,
{
    type Flag = <T::Decons as InitField<F>>::Flag;
    fn load<P>(
        cx: &dioxus_core::Scope<P>,
        o: &mut Option<crate::Shared<<F as crate::shared::Static>::Type, Self::Flag>>,
    ) {
        T::Decons::load(cx, o)
    }
}
impl<F: crate::shared::Static, T: crate::InductiveTuple> LoadFieldWrite<F> for [T; 1]
where
    T::Decons: LoadFieldWrite<F>,
{
    fn load_write(o: &mut Option<crate::Shared<<F as crate::shared::Static>::Type, Self::Flag>>) {
        T::Decons::load_write(o)
    }
}
pub struct LoadW<T>(pub T);
pub struct LoadRW<T>(pub T);

pub trait Implies<A> {}
impl Implies<()> for super::RW {}
impl Implies<()> for super::W {}
impl Implies<super::W> for super::W {}
impl Implies<super::W> for super::RW {}
impl Implies<super::RW> for super::RW {}
