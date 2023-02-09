pub mod assoc_type;

pub trait ShareableStruct: Sized {
    type Fields;
    type Actions;
    type Content;
}
pub trait FieldOf<S: ShareableStruct> {
    type RWType;
    type WType;
    const RW: Self::RWType;
    const W: Self::WType;
}
pub trait InitWith<O: super::InitType>: super::InitType {
    type Flag: super::InitType;
}
impl InitWith<()> for () {
    type Flag = ();
}
impl InitWith<super::W> for () {
    type Flag = super::W;
}
impl InitWith<super::RW> for () {
    type Flag = super::RW;
}
impl InitWith<()> for super::W {
    type Flag = super::W;
}
impl InitWith<super::W> for super::W {
    type Flag = super::W;
}
impl InitWith<super::RW> for super::W {
    type Flag = super::RW;
}
impl InitWith<()> for super::RW {
    type Flag = super::RW;
}
impl InitWith<super::W> for super::RW {
    type Flag = super::RW;
}
impl InitWith<super::RW> for super::RW {
    type Flag = super::RW;
}
pub trait ImpliesInitField<O: super::InitType>: super::InitType {}
impl<A: super::InitType, B: InitWith<A, Flag = A>> ImpliesInitField<B> for A {}

pub trait InitField<F> {
    type Flag: super::InitType;
}
impl<F> InitField<F> for () {
    type Flag = ();
}
impl<F, T: super::InductiveMarkerTuple> InitField<F> for T
where
    T::Step: InitField<F>,
    T::Base: InitField<F>,
    <T::Base as InitField<F>>::Flag: InitWith<<T::Step as InitField<F>>::Flag>,
{
    type Flag =
        <<T::Base as InitField<F>>::Flag as InitWith<<T::Step as InitField<F>>::Flag>>::Flag;
}
pub trait InitSubstruct<F> {
    type Actions;
    fn substruct_actions(&self) -> Self::Actions;
}
impl<F> InitSubstruct<F> for () {
    type Actions = ();
    fn substruct_actions(&self) {}
}
impl<F, T: super::InductiveMarkerTuple> InitSubstruct<F> for T
where
    T::Step: InitSubstruct<F> + Copy,
    T::Base: InitSubstruct<F> + Copy,
{
    type Actions = (
        <T::Base as InitSubstruct<F>>::Actions,
        <T::Step as InitSubstruct<F>>::Actions,
    );
    fn substruct_actions(&self) -> Self::Actions {
        (
            self.base().substruct_actions(),
            self.step().substruct_actions(),
        )
    }
}

pub trait WriteActions {}
impl WriteActions for () {}
impl<T: super::InductiveMarkerTuple> WriteActions for T
where
    T::Base: WriteActions,
    T::Step: WriteActions,
{
}

pub trait InitFieldAs<F, Flag: super::InitType>: InitField<F> {}
impl<F, T: InitField<F>, Flag: super::InitType> InitFieldAs<F, Flag> for T where
    <T as InitField<F>>::Flag: ImpliesInitField<Flag>
{
}

/// Create a `struct` definition for a global.
///
/// The idea is that each field of the struct will be stored in a separate global, and loaded only
/// when requested. The actions block describes possible ways of using the struct in terms of what
/// type of access ([`W`](crate::W) or [`RW`](crate::RW)) they need to fields of the struct.
///
/// The basic syntax is as follows:
/// ```
///     dioxus_shareables::shareable_struct! {
///         pub struct GlobalState {
///             a: usize = 8,
///             b: u16 = 12,
///             c: Vec<u8> = vec![],
///         }
///
///         action A impl pub ATrait = W[a] RW[b]; // Action A with equivalent trait ATrait
///         pub action B: BType = W[b] RW[a, c]; // Action B with equivalent type BType
///         action C: pub CType = RW[c]; // Action C with equivalent type CType
///         action D_ACTION impl pub D = W[c]; // Action D_ACTION with equivalent trait D.
///     }
/// ```
/// NOTE: fields in the struct must be `Send + Sync`
///
/// First we declare the struct itself, then "actions" which represent different views of the
/// struct. When we use the struct, we then have to declare which actions we need:
///
/// ```
///     # use dioxus::prelude::*;
///     # dioxus_shareables::shareable_struct! {
///     #     pub struct GlobalState {
///     #         a: usize = 8,
///     #         b: u16 = 12,
///     #         c: Vec<u8> = vec![],
///     #     }
///     #
///     #   action A impl pub ATrait = W[a] RW[b]; // Action A with equivalent trait ATrait
///     #   pub action B: BType = W[b] RW[a, c]; // Action B with equivalent type BType
///     #   action C: pub CType = RW[c]; // Action C with equivalent type CType
///     #   action D_ACTION impl pub D = W[c]; // Action D_ACTION with equivalent trait D.
///     # }
///     # #[allow(non_snake_case)]
///     fn Component(cx: Scope) -> Element {
///         let state = GlobalState::use_(cx, (A, B)); // Use GlobalState with actions A and B.
///         // ...
///         let b = *state.b().read(); // We can access field b because actions B includes it.
///         //...
///         # cx.render(rsx! {
///         #     div {
///         #       onmousedown: |_| { *state.a().write() += 1; },
///         #       onmouseover: |_| { *state.b().write() -= 3; }
///         #     }
///         # })
///     }
/// ```
///
/// Of course, there's not a lot of point to grouping shared variables into a type if we don't
/// implement some methods on the type. This is where the types on the actions come in:
/// ```
///     # use dioxus::prelude::*;
///     # dioxus_shareables::shareable_struct! {
///     #     pub struct GlobalState {
///     #         a: usize = 8,
///     #         b: u16 = 12,
///     #         c: Vec<u8> = vec![],
///     #     }
///     #
///     #   action A = W[a] RW[b]; // Action A with equivalent trait ATrait
///     #   pub action B: BType = W[b] RW[a, c]; // Action B with equivalent type BType
///     #   action C: pub CType = RW[c]; // Action C with equivalent type CType
///     #   action D_ACTION impl pub D = W[c]; // Action D_ACTION with equivalent trait D.
///     # }
///     impl GlobalState<CType> {
///         pub fn c_method(&self) {
///             // Do some stuff...
///         }
///     }
///     // Valid action markers implement GlobalStateActions:
///     impl<Actions: GlobalStateActions> GlobalState<Actions> {
///         // N.B. that D is the trait, not the actions constant:
///         pub fn clever_d_method(&self) where Actions: D {
///             let self_ = self.with_actions(D_ACTION); // We probably want to typecast
///                                                      // at the start of the method.
///             // ...
///         }
///     }
///     // ...
///     # #[allow(non_snake_case)]
///     fn Component(cx: Scope) -> Element {
///         let a_state = GlobalState::use_(cx, A);
///         let b_state = GlobalState::use_(cx, B);
///         let c_state = GlobalState::use_(cx, C);
///
///         // a_state.c_method(); // This will fail since `a_state` doesn't doesn't meet the RW[c] requirement.
///         // b_state.c_method(); // This will fail because the type is wrong.
///         b_state.as_ref().c_method(); // This works, but only if the type resolves correctly.
///         b_state.with_actions(C).c_method(); // This is guaranteed to work.
///         c_state.c_method(); // This works too.
///
///         // a_state.clever_d_method(); // Fails because a_state doesn't meet the W[c] requirement.
///         b_state.clever_d_method(); // This works.
///         c_state.clever_d_method(); // So does this.
///         # cx.render(rsx! { div {} })
///     }
/// ```
///
/// It's up to you where you prefer to typecast.
///
/// You don't need to declare actions in advance to use them; in particular, you may want to use
/// one-off action declarations on method declarations:
/// ```
///     # use dioxus::prelude::*;
///     # dioxus_shareables::shareable_struct! {
///     #     pub struct GlobalState {
///     #         a: usize = 8,
///     #         b: u16 = 12,
///     #         c: Vec<u8> = vec![],
///     #     }
///     #
///     #   action A = W[a] RW[b]; // Action A with equivalent trait ATrait
///     #   pub action B: BType = W[b] RW[a, c]; // Action B with equivalent type BType
///     #   action C: pub CType = RW[c]; // Action C with equivalent type CType
///     #   action D_ACTION impl pub D = W[c]; // Action D_ACTION with equivalent trait D.
///     # }
///     impl<Actions: GlobalStateActions> GlobalState<Actions> {
///         pub fn calculate_from_a_and_c(&self) -> usize where Actions:
///             AsGlobalStateActions<dioxus_shareables::struct_actions!{GlobalState<{RW[a] RW[c]}>}>
///         {
///             let self_ = self.with_actions(dioxus_shareables::struct_actions!(GlobalState(RW[a] RW[c])));
///             self_.a(); // you asked for it, you got it.
///             // ...
///             # 3
///         }
///     }
///     // ...
///     # #[allow(non_snake_case)]
///     fn Component(cx: Scope) -> Element {
///         let a_state = GlobalState::use_(cx, A);
///         let b_state = GlobalState::use_(cx, B);
///
///         // a_state.calculate_from_a_and_c(); // This will fail since `a_state` doesn't meet the RW[c] requirement.
///         b_state.calculate_from_a_and_c(); // This works, but only if the type resolves correctly.
///         # cx.render(rsx! { div {} })
///     }
/// ```
///
///
/// If you'd like, you can also organize your shared structure into substructures. A substructure
/// can be included in a larger shared structure by preceding the field name with a pipe like so:
/// ```
///     # use dioxus::prelude::*;
///     # dioxus_shareables::shareable_struct! {
///     #     pub struct GlobalState {
///     #         a: usize = 8,
///     #         b: u16 = 12,
///     #         c: Vec<u8> = vec![],
///     #     }
///     #
///     #   action A = W[a] RW[b]; // Action A with equivalent trait ATrait
///     #   pub action B: BType = W[b] RW[a, c]; // Action B with equivalent type BType
///     #   action C: pub CType = RW[c]; // Action C with equivalent type CType
///     #   action D_ACTION impl pub D = W[c]; // Action D_ACTION with equivalent trait D.
///     # }
///     # impl<Actions: GlobalStateActions> GlobalState<Actions> {
///     #     // N.B. that D is the trait, not the actions constant:
///     #     pub fn clever_d_method(&self) where Actions: D {
///     #         let self_ = self.with_actions(D_ACTION); // We probably want to typecast
///     #                                                  // at the start of the method.
///     #         // ...
///     #     }
///     # }
///     dioxus_shareables::shareable_struct! {
///         pub struct MoreGlobalState {
///             u: String = "more global? more state? which is it?!".into(),
///             v: u32 = 18,
///             |s: GlobalState,
///         }
///         action UVA = W[u] RW[v] |s[A]; // The included actions for s here must be a single
///                                        // ident which refers to a declared action for the
///                                        // given struct.
///         action UBC = W[u] |s[B]; // N.B.: The syntax doesn't change if B isn't in scope... B is
///                                  // accessed as an associated type of GlobalState.
///                                  // If you get errors involving `AssocType` bounds, this is a
///                                  // these |s[B] style bounds are the most likely candidtates.
///     }
///     // ...
///     # #[allow(non_snake_case)]
///     fn Component(cx: Scope) -> Element {
///         let mgs = MoreGlobalState::use_(cx, UBC);
///         mgs.s().clever_d_method(); // Works bcause action our mgs.s() was initialized with the
///                                    // `B` action.
///         // ...
///         # cx.render(rsx! { div {} })
///     }
/// ```
#[macro_export]
macro_rules! shareable_struct {
    (
        $(#[$meta:meta])*
        $v:vis struct $Struct:ident {
            $($fields:tt)*
        }
        $($actions:tt)*
    ) => {
        $crate::shareable_struct_parse_actions! {
            remaining_actions: {$($actions)*}
            vis: [$v]
            struct: [$Struct]
            meta: [$(#[$meta])*]
            fields: {$($fields)*}
            parsed_actions: []
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! shareable_struct_parse_actions {
    ( remaining_actions: {$av:vis action $ACTION:ident$(: $ATv:vis $ATy:ident)?$(impl $ATrv:vis $ATr:ident)? = $($r:tt)*}
      vis: $v:tt
      struct: $s:tt
      meta: $m:tt
      fields: $f:tt
      parsed_actions: $a:tt
    ) => {
        $crate::shareable_struct_parse_action_flags! {
            remaining_actions: {$($r)*}
            vis: $v
            struct: $s
            meta: $m
            fields: $f
            parsed_actions: $a
            action: [[$ACTION] vis: [$av] type: [$($ATv$ATy)?] trait: [$($ATrv$ATr)?]]
            w: []
            rw: []
            sub: []
        }
    };
    ( remaining_actions: {}
      vis: $v:tt
      struct: $s:tt
      meta: $m:tt
      fields: $f:tt
      parsed_actions: $a:tt
    ) => {
        $crate::shareable_struct_parse_fields! {
            remaining_fields: $f
            vis: $v
            struct: $s
            meta: $m
            standard_fields: []
            substruct_fields: []
            actions: $a
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! shareable_struct_parse_action_flags {
    ( remaining_actions: {W[$($w2:ident),*]$($r:tt)*}
      vis: $v:tt
      struct: $s:tt
      meta: $m:tt
      fields: $f:tt
      parsed_actions: $a:tt
      action: $ad:tt
      w: [$($w:tt)*]
      rw: $rw:tt
      sub: $sub:tt
    ) => {
        $crate::shareable_struct_parse_action_flags! {
            remaining_actions: {$($r)*}
            vis: $v
            struct: $s
            meta: $m
            fields: $f
            parsed_actions: $a
            action: $ad
            w: [$($w)*$(,$w2)*]
            rw: $rw
            sub: $sub
        }
    };
    ( remaining_actions: {RW[$($rw2:ident),*]$($r:tt)*}
      vis: $v:tt
      struct: $s:tt
      meta: $m:tt
      fields: $f:tt
      parsed_actions: $a:tt
      action: $ad:tt
      w: $w:tt
      rw: [$($rw:tt)*]
      sub: $sub:tt
    ) => {
        $crate::shareable_struct_parse_action_flags! {
            remaining_actions: {$($r)*}
            vis: $v
            struct: $s
            meta: $m
            fields: $f
            parsed_actions: $a
            action: $ad
            w: $w
            rw: [$($rw)*$(,$rw2)*]
            sub: $sub
        }
    };
    ( remaining_actions: {|$g:ident[$ga:ident]$($r:tt)*}
      vis: $v:tt
      struct: $s:tt
      meta: $m:tt
      fields: $f:tt
      parsed_actions: $a:tt
      action: $ad:tt
      w: $w:tt
      rw: $rw:tt
      sub: [$($sub:tt)*]
    ) => {
        $crate::shareable_struct_parse_action_flags! {
            remaining_actions: {$($r)*}
            vis: $v
            struct: $s
            meta: $m
            fields: $f
            parsed_actions: $a
            action: $ad
            w: $w
            rw: $rw
            sub: [$($sub)*sub_actions { sub: [$g] actions: [$ga] }]
        }
    };
    ( remaining_actions: {;$($r:tt)*}
      vis: $v:tt
      struct: $s:tt
      meta: $m:tt
      fields: $f:tt
      parsed_actions: [$($a:tt)*]
      action: [[$action:ident]$($ad:tt)*]
      w: $w:tt
      rw: $rw:tt
      sub: $sub:tt
    ) => {
        $crate::shareable_struct_parse_actions! {
            remaining_actions: {$($r)*}
            vis: $v
            struct: $s
            meta: $m
            fields: $f
            parsed_actions: [$($a)*
                action $action {
                    $($ad)*
                    w: $w
                    rw: $rw
                    sub: $sub
                }
            ]
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! shareable_struct_parse_fields {
    ( remaining_fields: {$fvis:vis $f:ident: $T:ty = $init:expr$(,$($r:tt)*)?}
      vis: $v:tt
      struct: $s:tt
      meta: $m:tt
      standard_fields: [$($ff:tt)*]
      substruct_fields: $g:tt
      actions: $a:tt
    ) => {
        $crate::shareable_struct_parse_fields! {
            remaining_fields: {$($($r)*)?}
            vis: $v
            struct: $s
            meta: $m
            standard_fields: [$($ff)*field $f { vis: [$fvis] type: [$T] init: [$init] }]
            substruct_fields: $g
            actions: $a
        }
    };
    ( remaining_fields: {}
      vis: $v:tt
      struct: $s:tt
      meta: $m:tt
      standard_fields: $f:tt
      substruct_fields: $g:tt
      actions: $a:tt
    ) => {
        $crate::shareable_struct_main! {
            vis: $v
            struct: $s
            meta: $m
            standard_fields: $f
            substruct_fields: $g
            actions: $a
        }
    };
    ( remaining_fields: {|$gvis:vis $g:ident: $T:ident$(::$Tc:ident)*$(,$($r:tt)*)?}
      vis: $v:tt
      struct: $s:tt
      meta: $m:tt
      standard_fields: $f:tt
      substruct_fields: $gg:tt
      actions: $a:tt
    ) => {
        $crate::shareable_struct_parse_substruct_path! {
            rest: [$($Tc)*]
            vis: $v
            struct: $s
            meta: $m
            standard_fields: $f
            substruct_fields: $gg
            remaining_fields: {$($($r)*)?}
            actions: $a
            substruct_field: [field $g { vis: [$gvis] }]
            head: [::]
            tail: $T
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! shareable_struct_parse_substruct_path {
    ( rest: [$Tn:ident$($Tc:tt)*]
      vis: $v:tt
      struct: $s:tt
      meta: $m:tt
      standard_fields: $f:tt
      substruct_fields: $gg:tt
      remaining_fields: $r:tt
      actions: $a:tt
      substruct_field: $g:tt
      head: [$($h:tt)*]
      tail: $Tl:ident
    ) => {
        $crate::shareable_struct_parse_substruct_path! {
            rest: [$($Tc)*]
            vis: $v
            struct: $s
            meta: $m
            standard_fields: $f
            substruct_fields: $gg
            remaining_fields: $r
            actions: $a
            substruct_field: $g
            head: [$($h)*$Tl::]
            tail: $Tn
        }
    };
    ( rest: []
      vis: $v:tt
      struct: $s:tt
      meta: $m:tt
      standard_fields: $f:tt
      substruct_fields: [$($gg:tt)*]
      remaining_fields: $r:tt
      actions: $a:tt
      substruct_field: [field $g:ident { vis: [$gvis:vis] }]
      head: [::$($h:tt)*]
      tail: $Tl:ident
    ) => {
        $crate::reexported::paste! {
            $crate::shareable_struct_parse_fields! {
                remaining_fields: $r
                vis: $v
                struct: $s
                meta: $m
                standard_fields: $f
                substruct_fields: [$($gg)*field $g { vis: [$gvis] struct: [$($h)*$Tl] actions: [$($h)*[<$Tl Actions>]] as_actions: [$($h)*[<As $Tl Actions>]] }]
                actions: $a
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! shareable_struct_main {
    ( vis: [$v:vis]
      struct: [$Struct:ident]
      meta: [$(#[$meta:meta])*]
      standard_fields: [
          $(field $f:ident {
              vis: [$fvis:vis]
              type: [$fT:ty]
              init: [$init:expr]
          })*
      ]
      substruct_fields: [
          $(field $g:ident {
              vis: [$gvis:vis]
              struct: [$gT:ty]
              actions:
              [$gAT:ty]
              as_actions: [$AgAT:ty]
          })*
      ]
      actions: [ $(
          action $ACTION:ident {
              vis: [$av:vis]
              type: [$($ATv:vis$ATy:ident)?]
              trait: [$($ATrv:vis$ATr:ident)?]
              w: [$(,$w:ident)*]
              rw: [$(,$rw:ident)*]
              sub: [$(sub_actions { sub: [$sa:ident] actions: [$saA:ident] })*]
          }
      )* ]
    ) => {
        $crate::reexported::paste! {
            $(#[$meta])*
            $v struct $Struct<__Actions: [<$Struct Actions>] = ()> {
                $($f: Option<$crate::Shared<$fT, <__Actions as [<$Struct Actions>]>::[<$f:camel Flag>]>>,)*
                $($g: $gT<<__Actions as [<$Struct Actions>]>::[<$g:camel Actions>]>,)*
                #[doc(hidden)]
                __actions_marker: std::marker::PhantomData<__Actions>,
            }
            impl<__Actions: [<$Struct Actions>]> $Struct<__Actions> {
                $v fn share(__a: __Actions) -> Self where __Actions: $crate::r#struct::WriteActions $(, <__Actions as [<$Struct Actions>]>::[<$g:camel Actions>]: $crate::r#struct::WriteActions)* {
                    #[allow(unused_mut)]
                    let mut self_ = Self {
                        $($f: None,)*
                        $($g: $gT::share([<$Struct Actions>]::[<$g _actions>](&__a)),)*
                        __actions_marker: std::marker::PhantomData,
                    };
                    $(
                        <__Actions::[<$f:camel Flag>] as $crate::InitType>::share_field(
                            &mut self_.$f,
                            <Self as $crate::r#struct::ShareableStruct>::Fields::[<$f:snake:upper>],
                        );
                    )*
                    self_
                }
                #[doc(hidden)]
                $v fn __uninit() -> Self {
                    Self {
                        $($f: None,)*
                        $($g: $gT::__uninit(),)*
                        __actions_marker: std::marker::PhantomData,
                    }
                }
                #[doc(hidden)]
                $v fn __init_in<P>(&mut self, cx: $crate::reexported::Scope<P>) {
                    $(
                        <__Actions::[<$f:camel Flag>] as $crate::InitType>::init_field(
                            cx,
                            &mut self.$f,
                            <Self as $crate::r#struct::ShareableStruct>::Fields::[<$f:snake:upper>],
                        );
                    )*
                    $(
                        self.$g.__init_in(cx);
                    )*
                }
                $v fn use_<'a, P>(cx: $crate::reexported::Scope<'a, P>, _: __Actions) -> &'a mut Self {
                    cx.use_hook(|| {
                        let mut self_ = Self::__uninit();
                        self_.__init_in(cx);
                        self_
                    })
                }
                $v fn with_actions<B: [<$Struct Actions>]>(&self, _: B) -> &$Struct<B>
                where __Actions: [<As $Struct Actions>]<B>
                {
                    // SAFETY:
                    //   * the layout of $Struct<F> does not depend on F.
                    //   * the [<As $Struct Actions>] trait guarantees that an initialized $Struct<__Actions>
                    //     has initialized all the fields that should be initialized in $Struct<B>
                    unsafe { std::mem::transmute(self) }
                }
                $($fvis fn $f(&self) -> &$crate::Shared<$fT, <__Actions as [<$Struct Actions>]>::[<$f:camel Flag>]> where <__Actions as [<$Struct Actions>]>::[<$f:camel Flag>]: $crate::Flag {
                    if let Some($f) = self.$f.as_ref() { $f }
                    else { unreachable!{} }
                })*
                $($gvis fn $g(&self) -> &$gT<<__Actions as [<$Struct Actions>]>::[<$g:camel Actions>]> {
                    &self.$g
                })*
            }
            #[doc = "Actions on a " $Struct]
            #[doc = "See [`dioxus_shareables::shareable_struct`] for more info"]
            /// An actions object describes a collection of field access types you might use
            /// together.
            $v trait [<$Struct Actions>]: 'static + Copy {
                $(type [<$f:camel Flag>]: $crate::InitType;)*
                $(
                    type [<$g:camel Actions>]: $gAT;
                    fn [<$g _actions>](&self) -> Self::[<$g:camel Actions>];
                )*
            }
            #[doc = "Marker trait for allowed conversions between `" [<$Struct Actions>] "` markers."]
            #[doc = "Implementing this yourself can lead to undefined behavior."]
            $v trait [<As $Struct Actions>]<B: [<$Struct Actions>]>: [<$Struct Actions>] {}
            impl<A: [<$Struct Actions>], B: [<$Struct Actions>]> [<As $Struct Actions>]<B> for A
            where
                A: 'static
                    $(+ $crate::r#struct::InitFieldAs<$crate::struct_assoc_type!{$Struct::Fields::$f}, <B as [<$Struct Actions>]>::[<$f:camel Flag>]>)*,
                $(<A as [<$Struct Actions>]>::[<$g:camel Actions>]: $AgAT<<B as [<$Struct Actions>]>::[<$g:camel Actions>]>,)*
            {
            }
            $(
                $av const $ACTION: $crate::struct_assoc_type!{$Struct::Actions::$ACTION} = <$Struct as $crate::r#struct::ShareableStruct>::Actions::$ACTION;
                $($ATv type $ATy = $crate::struct_assoc_type!{$Struct::Actions::$ACTION};)?
                $(
                    $ATrv trait $ATr: [<As $Struct Actions>]<$crate::struct_assoc_type!{$Struct::Actions::$ACTION}> {}
                    impl<T: [<As $Struct Actions>]<$crate::struct_assoc_type!{$Struct::Actions::$ACTION}>> $ATr for T {}
                )?
            )*
            impl<A: [<$Struct Actions>], B: [<As $Struct Actions>]<A>> AsRef<$Struct<A>> for $Struct<B> {
                fn as_ref(&self) -> &$Struct<A> {
                    // SAFETY:
                    //   * the layout of $Struct<F> does not depend on F.
                    //   * the [<As $Struct Actions>] trait guarantees that an initialized $Struct<B>
                    //     has initialized all the fields that should be initialized in $Struct<A>
                    unsafe { std::mem::transmute(self) }
                }
            }
            impl<A: [<$Struct Actions>], B: [<As $Struct Actions>]<A>> AsMut<$Struct<A>> for $Struct<B> {
                fn as_mut(&mut self) -> &mut $Struct<A> {
                    // SAFETY:
                    //   * the layout of $Struct<F> does not depend on F.
                    //   * the [<As $Struct Actions>] trait guarantees that an initialized $Struct<B>
                    //     has initialized all the fields that should be initialized in $Struct<A>
                    unsafe { std::mem::transmute(self) }
                }
            }
            #[allow(dead_code)]
            const _: () = {
                $v struct [<$Struct FieldData>];
                $v struct [<$Struct ActionData>];
                #[derive(Clone, Copy)]
                $v struct InitAs<F, A>(F, A);

                $v struct [<$Struct Content>] {
                    $($f: $crate::shared::Link<$fT>,)*
                    $($g: $crate::arcmap::ArcMap<<$gT as $crate::r#struct::ShareableStruct>::Content>,)*
                }
                impl Default for [<$Struct Content>] {
                    fn default() -> Self {
                        [<$Struct Content>] {
                            $($f: $crate::shared::Link::new({$init}),)*
                            $($g: $crate::arcmap::ArcMap::new(Default::default()),)*
                        }
                    }
                }

                impl<__Actions: [<$Struct Actions>]> $crate::r#struct::ShareableStruct for $Struct<__Actions> {
                    type Fields = [<$Struct FieldData>];
                    type Actions = [<$Struct ActionData>];
                    type Content = [<$Struct Content>];
                }

                static [<$Struct:snake:upper _STATIC>]:
                    std::sync::Mutex<Option<$crate::arcmap::ArcMap<[<$Struct Content>]>>> = std::sync::Mutex::new(None);

                $(  #[derive(Clone, Copy)]
                    $v struct [<$Struct FieldShareable $f:camel>];
                    impl $crate::shared::Static for [<$Struct FieldShareable $f:camel>] {
                        type Type = $fT;
                        fn _share(self) -> $crate::shared::Shared<Self::Type, $crate::W> {
                            let mut s = [<$Struct:snake:upper _STATIC>].lock().unwrap();
                            let a = match &mut *s {
                                Some(s) => s.clone(),
                                s @ None => s.insert($crate::arcmap::ArcMap::new(Default::default())).clone()
                            };
                            a.map(|c| &c.$f)._share()
                        }
                        fn _use_rw<'a, P>(self, cx: $crate::reexported::Scope<'a, P>) -> &'a mut $crate::shared::Shared<Self::Type, $crate::RW> {
                            let mut s = [<$Struct:snake:upper _STATIC>].lock().unwrap();
                            let a = match &mut *s {
                                Some(s) => s.clone(),
                                s @ None => s.insert($crate::arcmap::ArcMap::new(Default::default())).clone()
                            };
                            a.map(|c| &c.$f)._use_rw(cx)
                        }
                        fn _use_w<'a, P>(self, cx: $crate::reexported::Scope<'a, P>) -> &'a mut $crate::shared::Shared<Self::Type, $crate::W> {
                            let mut s = [<$Struct:snake:upper _STATIC>].lock().unwrap();
                            let a = match &mut *s {
                                Some(s) => s.clone(),
                                s @ None => s.insert($crate::arcmap::ArcMap::new(Default::default())).clone()
                            };
                            a.map(|c| &c.$f)._use_w(cx)
                        }
                    }
                    impl $crate::r#struct::FieldOf<$Struct> for [<$Struct FieldShareable $f:camel>] {
                        type WType = InitAs<[<$Struct FieldShareable $f:camel>], $crate::W>;
                        type RWType = InitAs<[<$Struct FieldShareable $f:camel>], $crate::RW>;
                        const W: Self::WType = InitAs([<$Struct FieldShareable $f:camel>], $crate::W);
                        const RW: Self::RWType = InitAs([<$Struct FieldShareable $f:camel>], $crate::RW);
                    }
                    impl $crate::r#struct::WriteActions for InitAs<[<$Struct FieldShareable $f:camel>], $crate::W> {}
                )*
                $(
                    #[derive(Clone, Copy)]
                    $v struct [<$Struct Substruct $g:camel>];
                )*

                impl [<$Struct FieldData>] {
                    $($fvis const [<$f:snake:upper>]: [<$Struct FieldShareable $f:camel>] = [<$Struct FieldShareable $f:camel>];)*
                }
                impl [<$Struct ActionData>] {
                    $(const $ACTION:
                        (
                            $(InitAs<[<$Struct FieldShareable $w:camel>], $crate::W>,)*
                            $(InitAs<[<$Struct FieldShareable $rw:camel>], $crate::RW>,)*
                            $(InitAs<[<$Struct Substruct $sa:camel>], $crate::struct_assoc_type!{$Struct::Substructs::$sa::Actions::$saA}>,)*
                        ) = (
                            $(InitAs([<$Struct FieldShareable $w:camel>], $crate::W),)*
                            $(InitAs([<$Struct FieldShareable $rw:camel>], $crate::RW),)*
                            $(InitAs([<$Struct Substruct $sa:camel>], <$crate::struct_assoc_type!{$Struct::Fields::$sa} as $crate::r#struct::ShareableStruct>::Actions::$saA),)*
                        );
                    )*
                }
                $crate::shareable_struct_init_as_fields!{
                    remaining_fields: [$($f)*]
                    struct: [$Struct]
                    init_as: [InitAs]
                    prev_fields: []
                    substruct_fields: [$(field $g { type: [$gAT] })*]
                }
                $crate::shareable_struct_init_as_substructs!{
                    remaining_fields: [$(field $g { type: [$gAT] })*]
                    struct: [$Struct]
                    init_as: [InitAs]
                    prev_fields: []
                    standard_fields: [$($f)*]
                }
                impl<A:
                    'static + Copy
                    $(+ $crate::r#struct::InitField<[<$Struct FieldShareable $f:camel>]>)*
                    $(+ $crate::r#struct::InitSubstruct<[<$Struct Substruct $g:camel>]>)*
                    > [<$Struct Actions>] for A
                where $(<A as $crate::r#struct::InitSubstruct<[<$Struct Substruct $g:camel>]>>::Actions: $gAT),* {
                    $(type [<$f:camel Flag>] = <A as $crate::r#struct::InitField<[<$Struct FieldShareable $f:camel>]>>::Flag;)*
                    $(
                        type [<$g:camel Actions>] = <A as $crate::r#struct::InitSubstruct<[<$Struct Substruct $g:camel>]>>::Actions;
                        fn [<$g _actions>](&self) -> Self::[<$g:camel Actions>] {
                            <A as $crate::r#struct::InitSubstruct<[<$Struct Substruct $g:camel>]>>::substruct_actions(self)
                        }
                    )*
                }

                $($crate::struct_assoc_type!{impl $Struct::Fields::$f for [<$Struct FieldData>] = [<$Struct FieldShareable $f:camel>]})*
                $($crate::struct_assoc_type!{impl $Struct::Fields::$g for [<$Struct FieldData>] = $gT})*
                $($crate::struct_assoc_type!{impl $Struct::Actions::$ACTION for [<$Struct ActionData>] =
                    (
                        $(InitAs<[<$Struct FieldShareable $w:camel>], $crate::W>,)*
                        $(InitAs<[<$Struct FieldShareable $rw:camel>], $crate::RW>,)*
                        $(InitAs<[<$Struct Substruct $sa:camel>], $crate::struct_assoc_type!{$Struct::Substructs::$sa::Actions::$saA}>,)*
                    )
                })*
            };
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! shareable_struct_init_as_fields {
    ( remaining_fields: [$f:ident$($r:ident)*]
      struct: [$Struct:ident]
      init_as:  [$IA:ident]
      prev_fields: [$($p:ident)*]
      substruct_fields: [$(field $g:ident {type: [$gAT:ty]})*]
    ) => {
        $crate::reexported::paste! {
            impl<A: $crate::InitType> $crate::r#struct::InitField<[<$Struct FieldShareable $f:camel>]> for InitAs<[<$Struct FieldShareable $f:camel>], A> {
                type Flag = A;
            }
            $(impl<A: $crate::InitType> $crate::r#struct::InitField<[<$Struct FieldShareable $f:camel>]> for InitAs<[<$Struct FieldShareable $r:camel>], A> {
                type Flag = ();
            })*
            $(impl<A: $crate::InitType> $crate::r#struct::InitField<[<$Struct FieldShareable $f:camel>]> for InitAs<[<$Struct FieldShareable $p:camel>], A> {
                type Flag = ();
            })*
            $(impl<A: $gAT> $crate::r#struct::InitField<[<$Struct FieldShareable $f:camel>]> for InitAs<[<$Struct Substruct $g:camel>], A> {
                type Flag = ();
            })*
        }
        $crate::shareable_struct_init_as_fields!{
            remaining_fields: [$($r)*]
            struct: [$Struct]
            init_as: [$IA]
            prev_fields: [$($p)*$f]
            substruct_fields: [$(field $g {type: [$gAT]})*]
        }
    };
    ( remaining_fields: []
      struct: $s:tt
      init_as: $IA:tt
      prev_fields: $p:tt
      substruct_fields: $g:tt
    ) => {};
}

#[doc(hidden)]
#[macro_export]
macro_rules! shareable_struct_init_as_substructs {
    ( remaining_fields: [field $g:ident {type: [$gAT:ty]}$(field $r:ident {type: [$rAT:ty]})*]
      struct: [$Struct:ident]
      init_as:  [$IA:ident]
      prev_fields: [$(field $p:ident {type: [$pAT:ty]})*]
      standard_fields: [$($f:ident)*]
    ) => {
        $crate::reexported::paste! {
            impl<A: $gAT> $crate::r#struct::InitSubstruct<[<$Struct Substruct $g:camel>]> for InitAs<[<$Struct Substruct $g:camel>], A> {
                type Actions = A;
                fn substruct_actions(&self) -> A {self.1}
            }
            $(impl<A: $rAT> $crate::r#struct::InitSubstruct<[<$Struct Substruct $g:camel>]> for InitAs<[<$Struct Substruct $r:camel>], A> {
                type Actions = ();
                fn substruct_actions(&self) -> () {}
            })*
            $(impl<A: $pAT> $crate::r#struct::InitSubstruct<[<$Struct Substruct $g:camel>]> for InitAs<[<$Struct Substruct $p:camel>], A> {
                type Actions = ();
                fn substruct_actions(&self) -> () {}
            })*
            $(impl<A: $crate::InitType> $crate::r#struct::InitSubstruct<[<$Struct Substruct $g:camel>]> for InitAs<[<$Struct FieldShareable $f:camel>], A> {
                type Actions = ();
                fn substruct_actions(&self) -> () {}
            })*
        }
        $crate::shareable_struct_init_as_substructs!{
            remaining_fields: [$(field $r {type: [$rAT]})*]
            struct: [$Struct]
            init_as:  [$IA]
            prev_fields: [$(field $p {type: [$pAT:ty]})*field $g {type: [$gAT]}]
            standard_fields: [$($f)*]
        }
    };
    ( remaining_fields: []
      struct: $s:tt
      init_as: $IA:tt
      prev_fields: $p:tt
      standard_fields: $g:tt
    ) => {};
}

/// Get actions on a struct.
///
/// For example `dioxus_shareables::struct_actions!(GlobalState<{W[a] RW[b]}>)` gives the correct
/// type for a `dioxus_shareables` struct with write access to field `a` and read-write access to
/// field `b`, and `dioxus_shareables::struct_actions!(GlobalState(W[a] RW[b]))` gives a
/// corresponding expression.
#[macro_export]
macro_rules! struct_actions {
    ($Struct:ident$(::$Struct_:ident)*<{$($ty:tt)*}>) => {
        $crate::struct_actions_! {
            unparsed: [$($ty)*]
            produce: ty
            struct: [$Struct$(::$Struct_)*]
        }
    };
    ($Struct:ident$(::$Struct_:ident)*($($ty:tt)*)) => {
        $crate::struct_actions_! {
            unparsed: [$($ty)*]
            produce: expr
            struct: [$Struct$(::$Struct_)*]
        }
    };
}
#[doc(hidden)]
#[macro_export]
macro_rules! struct_actions_ {
    (
        unparsed: []
        produce: $t:tt
        struct: [$Struct:path]
    ) => {
        ()
    };
    (
        unparsed: [W[$($w:ident)*]$($r:tt)*]
        produce: ty
        struct: [$($Struct:tt)*]
    ) => {
        (
            ($(
                <$crate::struct_assoc_type!($Struct::Fields::$w) as $crate::r#struct::FieldOf<$Struct>>::WType
            ),*),
            $crate::struct_actions_! {
                unparsed: [$($r)*]
                produce: ty
                struct: [$($Struct)*]
            }
        )
    };
    (
        unparsed: [RW[$($w:ident)*]$($r:tt)*]
        produce: ty
        struct: [$($Struct:tt)*]
    ) => {
        (
            ($(
                <$crate::struct_assoc_type!($Struct::Fields::$w) as $crate::r#struct::FieldOf<$Struct>>::RWType
            ),*),
            $crate::struct_actions_! {
                unparsed: [$($r)*]
                produce: ty
                struct: [$($Struct)*]
            }
        )
    };
    (
        unparsed: [W[$($w:ident)*]$($r:tt)*]
        produce: expr
        struct: [$($Struct:tt)*]
    ) => {
        (
            ($(
                <$crate::struct_assoc_type!($Struct::Fields::$w) as $crate::r#struct::FieldOf<$Struct>>::W
            ),*),
            $crate::struct_actions_! {
                unparsed: [$($r)*]
                produce: expr
                struct: [$($Struct)*]
            }
        )
    };
    (
        unparsed: [RW[$($w:ident)*]$($r:tt)*]
        produce: expr
        struct: [$($Struct:tt)*]
    ) => {
        (
            ($(
                <$crate::struct_assoc_type!($Struct::Fields::$w) as $crate::r#struct::FieldOf<$Struct>>::RW
            ),*),
            $crate::struct_actions_! {
                unparsed: [$($r)*]
                produce: expr
                struct: [$($Struct)*]
            }
        )
    };
}
