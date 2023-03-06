pub mod assoc_type;
/// Create a `struct` in which sharing can be controlled on the field level.
///
/// The idea is that each field of the struct will have separate update handles (i.e., be stored in
/// a separate (`Link`)[`crate::shared::Link`], and loaded only when requested. The actions block
/// describes possible ways of using the struct in terms of what type of access
/// ([`W`](crate::W) or [`RW`](crate::RW)) they need to fields of the struct.
///
/// The basic syntax is as follows:
/// ```
///     use dioxus_shareables::shareable_struct;
///     shareable_struct! {
///         pub static struct GlobalState {
///             a: usize = 8,
///             b: u16 = 12,
///             c: Vec<u8> = vec![],
///         }
///
///         action A: ATrait = { a: W, b: RW }; // Action A with an equivalent trait ATrait.
///         pub action B: BTrait = { b: W, a: RW, c: RW }; // B is visible outside the defining
///                                                        // module but BTrait is not.
///         action C: pub CTrait = { c: RW }; // CTrait is visible outside the defining module
///                                           // but C is not.
///         action _: pub D = { c: W }; // We define only the trait for D.
///         action E = { b: RW }; // And only the type for E.
///         action F = { A, a: RW }; // Acts like { a: RW, b: RW };
///     }
/// ```
/// NOTE: fields in the struct must be `Send + Sync`
///
/// First we declare the struct itself, then "actions" which represent different views of the
/// struct. When we use the struct, we then have to declare which actions we need:
///
/// ```
///     # use dioxus::prelude::*;
///     # use dioxus_shareables::shareable_struct;
///     # shareable_struct! {
///     #   pub static struct GlobalState {
///     #       a: usize = 8,
///     #       b: u16 = 12,
///     #       c: Vec<u8> = vec![],
///     #   }
///     #
///     #   action A: ATrait = { a: W, b: RW }; // Action A with an equivalent trait ATrait.
///     #   pub action B: BTrait = { b: W, a: RW, c: RW }; // B is visible outside the defining
///     #                                                  // module but BTrait is not.
///     #   action C: pub CTrait = { c: RW }; // CTrait is visible outside the defining module
///     #                                     // but C is not.
///     #   action _: pub D = { c: W }; // We define only the trait for D.
///     #   action E = { b: RW }; // And only the type for E.
///     #   action F = { A, a: RW }; // Acts like { a: RW, b: RW };
///     # }
///     # #[allow(non_snake_case)]
///     //...
///     fn Component1(cx: Scope) -> Element {
///         let state = GlobalState::use_::<(A,B), _>(cx); // Use GlobalState with actions A and B.
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
///     # use dioxus_shareables::shareable_struct;
///     # shareable_struct! {
///     #     pub static struct GlobalState {
///     #         a: usize = 8,
///     #         b: u16 = 12,
///     #         c: Vec<u8> = vec![],
///     #     }
///     #
///     #   action A: ATrait = { a: W, b: RW }; // Action A with an equivalent trait ATrait.
///     #   pub action B: BTrait = { b: W, a: RW, c: RW }; // B is visible outside the defining
///     #                                                  // module but BTrait is not.
///     #   action C: pub CTrait = { c: RW }; // CTrait is visible outside the defining module
///     #                                     // but C is not.
///     #   action _: pub D = { c: W }; // We define only the trait for D.
///     #   action E = { b: RW }; // And only the type for E.
///     #   action F = { A, a: RW }; // Acts like { a: RW, b: RW };
///     # }
///     //...
///     impl GlobalState<C> {
///         pub fn c_method(&self) {
///             // Do some stuff...
///         }
///     }
///     // Valid action markers implement GlobalStateActions:
///     impl<Actions: GlobalStateActions> GlobalState<Actions> {
///         // N.B. that D is the trait, not the actions constant:
///         pub fn clever_d_method(&self) where Actions: D {
///             let self_ = Actions::typecast(self); // Trait D provides this typecast method.
///             // ...
///         }
///     }
///     // ...
///     # #[allow(non_snake_case)]
///     fn Component2(cx: Scope) -> Element {
///         let a_state: &GlobalState<A> = GlobalState::use_(cx);
///         let b_state: &GlobalState<B> = GlobalState::use_(cx);
///         let c_state: &GlobalState<C> = GlobalState::use_(cx);
///
///         c_state.c_method(); // This is fine, since it's the type we defined the function on.
///         // b_state.c_method(); // This will fail because the type is wrong.
///         b_state.as_ref().c_method(); // This works, but only if the type resolves correctly.
///         b_state.with_actions::<C>().c_method(); // This is guaranteed to work.
///         // a_state.as_ref().c_method();            // These will fail since `a_state` doesn't
///         // a_state.with_actions::<C>().c_method(); // meet the `c: RW` requirement.
///
///         // a_state.clever_d_method(); // Fails because a_state doesn't meet the `c: W` requirement.
///         b_state.clever_d_method(); // This works.
///         c_state.clever_d_method(); // So does this.
///         # cx.render(rsx! { div {} })
///     }
/// ```
///
/// It's up to you where you prefer to typecast.
///
/// You don't need to declare actions in advance to use them; in particular, you may decide you
/// want to use one-off action declarations on method declarations:
/// ```
///     # use dioxus::prelude::*;
///     # use dioxus_shareables::shareable_struct;
///     # shareable_struct! {
///     #     pub static struct GlobalState {
///     #         a: usize = 8,
///     #         b: u16 = 12,
///     #         c: Vec<u8> = vec![],
///     #     }
///     #
///     #   action A: ATrait = { a: W, b: RW }; // Action A with an equivalent trait ATrait.
///     #   pub action B: BTrait = { b: W, a: RW, c: RW }; // B is visible outside the defining
///     #                                                  // module but BTrait is not.
///     #   action C: pub CTrait = { c: RW }; // CTrait is visible outside the defining module
///     #                                     // but C is not.
///     #   action _: pub D = { c: W }; // We define only the trait for D.
///     #   action E = { b: RW }; // And only the type for E.
///     #   action F = { A, a: RW }; // Acts like { a: RW, b: RW };
///     # }
///     // ...
///     use dioxus_shareables::struct_actions;
///     impl<Actions: GlobalStateActions> GlobalState<Actions> {
///         # #[must_use]
///         pub fn calculate_from_a_and_c(&self) -> usize
///         where
///             Self: AsRef<GlobalState<struct_actions!{GlobalState {a: RW, c: RW}}>>
///         {
///             let self_ = self.as_ref();
///             self_.a();
///             // ...
///             # 3
///         }
///     }
///     // ...
///     # #[allow(non_snake_case)]
///     fn Component3(cx: Scope) -> Element {
///         let a_state = GlobalState::use_::<A, _>(cx);
///         let b_state = GlobalState::use_::<B, _>(cx);
///
///         // a_state.calculate_from_a_and_c(); // This will fail since `a_state` doesn't meet the RW[c] requirement.
///         b_state.calculate_from_a_and_c(); // This works, but only if the type resolves correctly.
///         # cx.render(rsx! { div {} })
///     }
/// ```
///
///
/// If you'd like, you can also organize your shared structure into substructures. One generally
/// wants to declare a Substruct without the `static` keyword (so that there is no global instance
/// of the type, just the ones that appear as substructures).
/// ```
///     # use dioxus::prelude::*;
///     use dioxus_shareables::shareable_struct;
///     shareable_struct! {
///         pub struct Substruct {
///             a: usize = 8,
///             b: u16 = 12,
///             c: Vec<u8> = vec![],
///         }
///     
///       action A = {a: W, b: RW};
///       action B = {a: RW, b: W, c: RW};
///       action C = {c: RW};
///       action _: pub D = {c: W};
///     }
///     impl<Actions: SubstructActions> Substruct<Actions> {
///         pub fn clever_d_method(&self)
///         where
///             Actions: D,
///         {
///             let self_ = Actions::typecast(self);
///             // ...
///         }
///     }
///     // ...
///     shareable_struct! {
///         pub static struct ParentStruct {
///             u: String = "Some silly string...".into(),
///             v: u32 = 18,
///             |s: Substruct,
///             |t: Substruct, // N.B. `s` and `t` will point to different instances of `Substruct`.
///         }
///         action UVA = {u: W, v: RW, |s: { a: RW, c: W }};
///         action UBC = {u: W, |t: { A, B }}; // N.B. you can refer to actions `A` and `B` even though
///                                            // they aren't in scope. (Since they were defined in the
///                                            // initial definition of `Substruct`).
///     }
///     // ...
///     #[allow(non_snake_case)]
///     fn Component(cx: Scope) -> Element {
///         let mgs = ParentStruct::use_::<UVA, _>(cx);
///         mgs.s.clever_d_method(); // Works bcause action our mgs.s was initialized with the
///                                  // `c: RW` (this is part of action `B`) and this implies
///                                  // action `D`.
///         // ...
///         # cx.render(rsx! { div {} })
///     }
/// ```
///
/// More complicated relationships between shareable structs (for example a collection of shareable
/// structs contained in one shareable struct) can be acheived using the associated Shareable type.
///
/// ```
///     # use dioxus::prelude::*;
///     use dioxus_shareables::{shareable_struct, struct_actions, struct_assoc_type};
///     shareable_struct! {
///       pub struct Todo {
///           priority: u32 = 0,
///           desc: String = "<DESCRIPTION HERE>".into(),
///           tags: Vec<String> = vec![],
///       }
///       action CreateTodo = { priority: W, desc: W };
///       action ShowTodo = { priority: RW, desc: RW, tags: RW };
///     }
///     // ...
///     shareable_struct! {
///         pub static struct TodoList {
///             title: String = "TODO".into(),
///             author: String = "You".into(),
///             todos: Vec<struct_assoc_type!(Todo::Shareable)> = vec![],
///         }
///         action ReadAll = { title: RW, author: RW, todos: RW };
///     }
///     // ...
///     impl<A: TodoListActions> TodoList<A> {
///         fn new_todo(&self, priority: u32, desc: String)
///         where
///             Self: AsRef<TodoList<struct_actions!(TodoList { todos: W })>>
///         {
///             let new_todo = <struct_assoc_type!{Todo::Shareable}>::default();
///             let e: Todo<CreateTodo> = new_todo.share();
///             *e.desc().write() = desc;
///             *e.priority().write() = priority;
///             self.as_ref().todos().write().push(new_todo);
///         }
///     }
///     // ...
///     # #[allow(non_snake_case)]
///     fn TodoListComponent(cx: Scope) -> Element {
///         let list: &TodoList<ReadAll> = TodoList::use_(cx);
///         let title = list.title().read();
///         let author = list.author().read();
///         cx.render(rsx! {
///             div { class: "todolisttitle", "{title}" }
///             div { class: "todolistauthor", "{author}" }
///             list.todos().read().iter().cloned().map(|todo| rsx! { TodoComponent { todo: todo } })
///         })
///     }
///     // ...
///     # #[allow(non_snake_case)]
///     #[inline_props]
///     fn TodoComponent(cx: Scope, todo: struct_assoc_type!(Todo::Shareable)) -> Element {
///         let todo: &Todo<ShowTodo> = todo.use_(cx); // = todo.use_::<Todo<ShowTodo>>(cx);
///         // ...
///         # cx.render(rsx! { div {} })
///     }
/// ```
#[macro_export]
#[allow(clippy::module_name_repetitions)]
macro_rules! shareable_struct {
    (
        $(#[$meta:meta])*
        $v:vis $(static$(@$static:tt)?)? struct $Struct:ident {
            $($fields:tt)* // $(vis ident: ty = expr,)*
        }
        $($actions:tt)* // $(vis action IDENT$($ident)?
    ) => {
        $crate::__shareable_struct_parse_fields! {
            unparsed_fields: [$($fields)*]
            vis: [$v]
            static: [$(static$($static)?)?]
            attr: [$(#[$meta])*]
            struct: $Struct
            fields: []
            substructs: []
            actions: [$($actions)*]
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __shareable_struct_parse_fields {
    ( unparsed_fields: [$fvis:vis $f:ident: $fty:ty = $init:expr$(,$($unparsed:tt)*)?]
      vis: $vis:tt
      static: $static:tt
      attr: $attr:tt
      struct: $Struct:ident
      fields: [$([
          vis: $pvis:tt
          name: $p:ident
          type: $pty:tt
          init: $pinit:tt
          other: [$($pp:ident)*]
          substructs: $ps:tt
          actions: []
      ])*]
      substructs: [
        $([
            vis: $svis:tt
            name: $s:ident
            ty: $sty:tt
            fields: [$($sf:ident)*]
            other: $sos:tt
            actions: []
        ])*
      ]
      actions: $actions:tt
    ) => {
        $crate::__shareable_struct_parse_fields! {
            unparsed_fields: [$($($unparsed)*)?]
            vis: $vis
            static: $static
            attr: $attr
            struct: $Struct
            fields: [
                $([
                    vis: $pvis
                    name: $p
                    type: $pty
                    init: $pinit
                    other: [$($pp)*$f]
                    substructs: $ps
                    actions: []
                ])*
                [
                    vis: [$fvis]
                    name: $f
                    type: [$fty]
                    init: [$init]
                    other: [$($p)*]
                    substructs: [$($s)*]
                    actions: []
                ]
            ]
            substructs: [
                $([
                    vis: $svis
                    name: $s
                    ty: $sty
                    fields: [$($sf)*$f]
                    other: $sos
                    actions: []
                ])*
            ]
            actions: $actions
        }
    };
    ( unparsed_fields: [|$fvis:vis $f:ident: $fty:ty$(,$($unparsed:tt)*)?]
      vis: $vis:tt
      static: $static:tt
      attr: $attr:tt
      struct: $Struct:ident
      fields: [$([
          vis: $pvis:tt
          name: $p:ident
          type: $pty:tt
          init: $pinit:tt
          other: $pof:tt
          substructs: [$($ps:ident)*]
          actions: []
      ])*]
      substructs: [$([
          vis: $svis:tt
          name: $s:ident
          ty: $sty:tt
          fields: $sf:tt
          other: [$($sos:ident)*]
          actions: []
      ])*]
      actions: $actions:tt
    ) => {
        $crate::__shareable_struct_parse_fields! {
            unparsed_fields: [$($($unparsed)*)?]
            vis: $vis
            static: $static
            attr: $attr
            struct: $Struct
            fields: [$([
                vis: $pvis
                name: $p
                type: $pty
                init: $pinit
                other: $pof
                substructs: [$($ps)*$f]
                actions: []
            ])*]
            substructs: [
                $([
                    vis: $svis
                    name: $s
                    ty: $sty
                    fields: $sf
                    other: [$($sos)*$f]
                    actions: []
                ])*
                [
                    vis: [$fvis]
                    name: $f
                    ty: [$fty]
                    fields: [$($p)*]
                    other: [$($s)*]
                    actions: []
                ]
            ]
            actions: $actions
        }
    };
    ( unparsed_fields: []
      vis: $vis:tt
      static: $static:tt
      attr: $attr:tt
      struct: $Struct:ident
      fields: $fields:tt
      substructs: $substructs:tt
      actions: $actions:tt
    ) => {
        $crate::__shareable_struct_parse_actions! {
            unparsed_actions: $actions
            vis: $vis
            static: $static
            attr: $attr
            struct: $Struct
            fields: $fields
            substructs: $substructs
            actions: []
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __shareable_struct_parse_actions {
    ( unparsed_actions: [
        action _: $atrvis:vis $atr:ident = {$($abod:tt)*};
        $($unparsed:tt)*
      ]
      vis: $vis:tt
      static: $static:tt
      attr: $attr:tt
      struct: $Struct:ident
      fields: [$([
              vis: $fvis:tt
              name: $f:ident
              type: $fty:tt
              init: $finit:tt
              other: $otherf:tt
              substructs: $substruct:tt
              actions: [$($fact:tt)*]
          ])*]
      substructs: [$([
              vis: $svis:tt
              name: $s:ident
              ty: $sty:tt
              fields: $field:tt
              other: $others:tt
              actions: [$($sact:tt)*]
          ])*]
      actions: [$([
          ty: [$($oatyvis:tt$oaty:ident)?]
          tr: $oatr:tt
          actions: $otheraions:tt
          fields: $ofields:tt
          substructs: $osubstructs:tt
          other: [$($oothera:ident)*]
      ])*]
    ) => {
        $crate::__shareable_struct_parse_actions! {
            unparsed_actions: [$($unparsed)*]
            vis: $vis
            static: $static
            attr: $attr
            struct: $Struct
            fields: [$([
                    vis: $fvis
                    name: $f
                    type: $fty
                    init: $finit
                    other: $otherf
                    substructs: $substruct
                    actions: [$($fact)*]
                ])*]
            substructs: [$([
                    vis: $svis
                    name: $s
                    ty: $sty
                    fields: $field
                    other: $others
                    actions: [$($sact)*]
                ])*]
            actions: [
                $([
                    ty: [$($oatyvis$oaty)?]
                    tr: $oatr
                    actions: $otheraions
                    fields: $ofields
                    substructs: $osubstructs
                    other: [$($oothera)*]
                ])*
                [
                    ty: []
                    tr: [[$atrvis]$atr]
                    actions: [$crate::struct_actions!{$Struct{$($abod)*}}]
                    fields: [$($f)*]
                    substructs: [$($s)*]
                    other: [$($($oaty)?)*]
                ]
            ]
        }
    };
    ( unparsed_actions: [
        $atyvis:vis action $aty:ident$(: $atrvis:vis $atr:ident)? = {$($abod:tt)*};
        $($unparsed:tt)*
      ]
      vis: $vis:tt
      static: $static:tt
      attr: $attr:tt
      struct: $Struct:ident
      fields: [$([
             vis: $fvis:tt
             name: $f:ident
             type: $fty:tt
             init: $finit:tt
             other: $otherf:tt
             substructs: $substruct:tt
             actions: [$($fact:tt)*]
          ])*]
      substructs: [$([
             vis: $svis:tt
             name: $s:ident
             ty: $sty:tt
             fields: $field:tt
             other: $others:tt
             actions: [$($sact:tt)*]
         ])*]
      actions: [$([
          ty: [$($oatyvis:tt$oaty:ident)?]
          tr: $oatr:tt
          actions: $otheraions:tt
          fields: $ofields:tt
          substructs: $osubstructs:tt
          other: [$($oothera:ident)*]
      ])*]
    ) => {
        $crate::__shareable_struct_parse_actions! {
            unparsed_actions: [$($unparsed)*]
            vis: $vis
            static: $static
            attr: $attr
            struct: $Struct
            fields: [$([
                    vis: $fvis
                    name: $f
                    type: $fty
                    init: $finit
                    other: $otherf
                    substructs: $substruct
                    actions: [$($fact)*$aty]
                ])*]
            substructs: [$([
                    vis: $svis
                    name: $s
                    ty: $sty
                    fields: $field
                    other: $others
                    actions: [$($sact)*$aty]
                ])*]
            actions: [
                $([
                    ty: [$($oatyvis$oaty)?]
                    tr: $oatr
                    actions: $otheraions
                    fields: $ofields
                    substructs: $osubstructs
                    other: [$($oothera)*$aty]
                ])*
                [
                    ty: [[$atyvis]$aty]
                    tr: [$([$atrvis]$atr)?]
                    actions: [$crate::struct_actions!{$Struct{$($abod)*}}]
                    fields: [$($f)*]
                    substructs: [$($s)*]
                    other: [$($($oaty)?)*]
                ]
            ]
        }
    };
    ( unparsed_actions: []
      vis: $vis:tt
      static: $static:tt
      attr: $attr:tt
      struct: $Struct:ident
      fields: [$([
               vis: $fvis:tt
               name: $f:ident
               type: $fty:tt
               init: $finit:tt
               other: [$($otherf:ident)*]
               substructs: [$($substruct:ident)*]
               actions: [$($fact:ident)*]
          ])*]
      substructs: [$([
              vis: $svis:tt
              name: $s:ident
              ty: $sty:tt
              fields: [$($field:ident)*]
              other: [$($others:ident)*]
              actions: [$($sact:ident)*]
          ])*]
      actions: [$([
             ty: [$($atyvis:tt $aty:ident)?]
             tr: $atr:tt
             actions: $abody:tt
             fields: [$($afield:ident)*]
             substructs: [$($asubstruct:ident)*]
             other: [$($othera:ident)*]
          ])*]
    ) => {
        $crate::reexported::paste! {
            $crate::__shareable_struct_main! {
                vis: $vis
                static: [<$Struct:snake:upper _ STATIC>]$static
                attr: $attr
                struct: $Struct
                actions: [<$Struct Actions>]
                as_actions: [<As $Struct Actions>]
                content: [<$Struct Content>]
                fielddata: [<$Struct FieldData>]
                substructdata: [<$Struct SubstructData>]
                actiondata: [<$Struct ActionData>]
                asinit: [<$Struct InitAs>]
                fields: [$([
                    vis: $fvis
                    name: $f
                    marker: [<$Struct __ $f:camel>]
                    type: $fty
                    init: $finit
                    other: [$([<$Struct __ $otherf:camel>])*]
                    substructs: [$([<$Struct __ $substruct:camel>])*]
                    actions: [$([<$Struct Actions__ $fact>])*]
                ])*]
                substructs: [$([
                    vis: $svis
                    name: $s
                    marker: [<$Struct __ $s:camel>]
                    ty: $sty
                    fields: [$([<$Struct __ $field:camel>])*]
                    other: [$([<$Struct __ $others:camel>])*]
                    actions: [$([<$Struct Actions__ $sact>])*]
                ])*]
                actions: [$([
                    ty: [$($atyvis $aty [<$Struct Actions__ $aty>])?]
                    tr: $atr
                    actions: $abody
                    fields: [$([<$Struct __ $afield:camel>])*]
                    substructs: [$([<$Struct __ $asubstruct:camel>])*]
                    other: [$($othera)*]
                ])*]
            }
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! __shareable_struct_main {
    (if [] {$($_:tt)*}$(else {$($t:tt)*})?) => {$($t)*};
    (if [$($_:tt)*] {$($t:tt)*}$(else {$($__:tt)*})?) => {$($t)*};
    ( vis: [$vis:vis]
      static: $STATIC:ident$is_static:tt
      attr: [$($attr:tt)*]
      struct: $Struct:ident
      actions: $StructActions:ident
      as_actions: $AsStructActions:ident
      content: $StructContent:ident
      fielddata: $StructFieldData:ident
      substructdata: $StructSubstructData:ident
      actiondata: $StructActionData:ident
      asinit: $StructInitAs:ident
      fields: [$([
               vis: [$fvis:vis]
               name: $f:ident
               marker: $fdata:ident
               type: [$fty:ty]
               init: [$finit:expr]
               other: [$($otherf:ident)*]
               substructs: [$($substruct:ident)*]
               actions: [$($fact:ident)*]
          ])*]
      substructs: [$([
             vis: [$svis:vis]
             name: $s:ident
             marker: $sdata:ident
             ty: [$sty:ty]
             fields: [$($field:ident)*]
             other: [$($others:ident)*]
             actions: [$($sact:ident)*]
          ])*]
      actions: [$([
             ty: [$([$atyvis:vis]$aty:ident$atymarker:ident)?]
             tr: [$([$atrvis:vis]$atr:ident)?]
             actions: [$abody:ty]
             fields: [$($afield:ident)*]
             substructs: [$($asubstruct:ident)*]
             other: [$($othera:ident)*]
          ])*]
    ) => {
        $($attr)*
        #[repr(C)] // This will guarantee that the fields aren't reordered based on the Actions type.
                   // Since align(Struct<__Actions>) is just align(Shared<_, _>) (which in turn is
                   // the align of each of the fields), this shouldn't end up adding padding.
        $vis struct $Struct<__Actions: $StructActions = ()> {
            $($f: Option<$crate::shared::Shared<$fty, <__Actions as $crate::r#struct::InitField<$Struct,$crate::struct_assoc_type!($Struct::Fields::$f)>>::Flag>>,)*
            $($svis $s:
                <
                    <__Actions as
                        $crate::r#struct::InitSubstruct<
                            $Struct,
                            $crate::struct_assoc_type!($Struct::Fields::$s)
                        >
                    >::Actions as
                        $crate::r#struct::ActionsFor<
                            <$crate::struct_assoc_type!($Struct::Fields::$s) as
                                $crate::r#struct::SubstructOf<$Struct>
                            >::Type
                        >
                >::WithActions,
            )*
            #[doc(hidden)]
            __actions: std::marker::PhantomData<__Actions>,
        }
        impl $Struct {
            $crate::__shareable_struct_main! {
                if $is_static {
                    #[must_use]
                    $vis fn use_<__Actions: $StructActions, P>(cx: $crate::reexported::Scope<P>) -> &$Struct<__Actions> {
                        let id = cx.scope_id().0;
                        cx.use_hook(||
                            <__Actions as $crate::r#struct::ActionsFor<Self>>::use_(
                                (id, cx.schedule_update()),
                                <Self as $crate::r#struct::Static>::get_static()
                            )
                        )
                    }
                    #[must_use]
                    $vis fn share<__Actions: $StructActions>() -> $Struct<__Actions>
                    where
                        __Actions: $crate::r#struct::ActionsFor<Self, WithActions=$Struct<__Actions>>
                                    + $crate::r#struct::WriteActionsFor<Self>
                    {
                        <__Actions as $crate::r#struct::WriteActionsFor<Self>>::share(
                            <Self as $crate::r#struct::Static>::get_static()
                        )
                    }
                }
            }
            #[must_use]
            $vis fn new() -> $crate::arcmap::ArcMap<<Self as $crate::r#struct::ShareableStruct>::Content> {
                Default::default()
            }
        }
        impl<__Actions: $StructActions> $Struct<__Actions> {
            $( fn $f(&self) -> &$crate::shared::Shared<$fty, <__Actions as $crate::r#struct::InitField<$Struct, $crate::struct_assoc_type!($Struct::Fields::$f)>>::Flag> {
                   unimplemented!()
               }
            )*
            fn with_actions<__ImpliedActions: $StructActions>(&self) -> &$Struct<__ImpliedActions>
            where
                Self: std::convert::AsRef<$Struct<__ImpliedActions>>
            {
                self.as_ref()
            }
        }
        $vis trait $StructActions:
            Default
                $(+ $crate::r#struct::InitField<$Struct,$crate::struct_assoc_type!($Struct::Fields::$f)>)*
                $(+ $crate::r#struct::InitSubstruct<$Struct,$crate::struct_assoc_type!($Struct::Fields::$s)>)*
        {
        }
        impl<__Actions: $StructActions> $crate::r#struct::ShareableStructWithActions for $Struct<__Actions> {
            type Base = $Struct;
            type Actions = __Actions;
        }
        $(
            $($atyvis type $aty = $crate::struct_assoc_type!($Struct::Actions::$aty);)?
            $($atrvis trait $atr: $StructActions {
                fn typecast(s: &$Struct<Self>) -> &$Struct<$abody>;
            }
            impl<__Actions: $StructActions> $atr for __Actions where $Struct<__Actions>: std::convert::AsRef<$Struct<$abody>> {
                fn typecast(s: &$Struct<Self>) -> &$Struct<$abody> {
                    s.as_ref()
                }
            }
            )*
        )*
        const _: () = {
            $crate::__shareable_struct_main!(if $is_static {
                static $STATIC: std::sync::Mutex<Option<$crate::arcmap::ArcMap<$StructContent>>> = std::sync::Mutex::new(None);
                impl $crate::r#struct::Static for $Struct {
                    fn r#static() -> &'static std::sync::Mutex<Option<$crate::arcmap::ArcMap<$StructContent>>> { &$STATIC }
                }
            });
            $vis struct $StructContent {
                $($f: $crate::shared::Link<$fty>,)*
                $($s: <$sty as $crate::r#struct::ShareableStruct>::Content,)*
            }
            impl $crate::r#struct::Content for $StructContent {
                type For = $Struct;
            }
            impl Default for $StructContent {
                fn default() -> Self {
                    Self {
                        $($f: $crate::shared::Link::new($finit),)*
                        $($s: Default::default(),)*
                    }
                }
            }
            $vis struct $StructFieldData;
            $(
                $vis struct $fdata;
                $crate::struct_assoc_type!(impl $Struct::Fields::$f for $StructFieldData = $fdata);
                impl $crate::r#struct::FieldOf<$Struct> for $fdata {
                    type Type = $fty;
                    fn get_field(
                        __field: $crate::arcmap::ArcMap<$StructContent>
                    ) -> $crate::arcmap::ArcMap<$crate::shared::Link<$fty>> {
                        __field.map(|__field| &__field.$f)
                    }
                }
                impl<__InitFlag: $crate::r#struct::InitFlag> $crate::r#struct::InitField<$Struct, $fdata> for $StructInitAs<$fdata, __InitFlag> {
                    type Flag = __InitFlag;
                }
                impl<_F> $crate::r#struct::Simple for $StructInitAs<$fdata,_F> {}
                impl<_A, _F> $crate::r#struct::Append<_A> for $StructInitAs<$fdata,_F> {
                    type Appended = ($StructInitAs<$fdata,_F>, _A);
                }
                impl<_F> $crate::r#struct::PiecewiseSimplify<()> for $StructInitAs<$fdata,_F> {
                    type Combined = $StructInitAs<$fdata,_F>;
                    type Remainder = ();
                }
                impl<_F: $crate::r#struct::InitFlag, _G: $crate::r#struct::CombineFlag<_F>> $crate::r#struct::PiecewiseSimplify<$StructInitAs<$fdata,_F>> for $StructInitAs<$fdata,_G> {
                    type Combined = $StructInitAs<$fdata, _G::Combined>;
                    type Remainder = ();
                }
                $(
                    impl<__InitFlag: $crate::r#struct::InitFlag> $crate::r#struct::InitField<$Struct, $otherf> for $StructInitAs<$fdata, __InitFlag> {
                        type Flag = ();
                    }
                    impl<_F, _G> $crate::r#struct::PiecewiseSimplify<$StructInitAs<$fdata, _F>> for $StructInitAs<$otherf, _G> {
                        type Combined = $StructInitAs<$otherf, _G>;
                        type Remainder = $StructInitAs<$fdata, _F>;
                    }
                )*
                $(
                    impl<__InitFlag: $crate::r#struct::InitFlag> $crate::r#struct::InitSubstruct<$Struct, $substruct> for $StructInitAs<$fdata, __InitFlag> {
                        type Actions = ();
                    }
                    impl<_F, _G> $crate::r#struct::PiecewiseSimplify<$StructInitAs<$fdata, _F>> for $StructInitAs<$substruct, _G> {
                        type Combined = $StructInitAs<$substruct, _G>;
                        type Remainder = $StructInitAs<$fdata, _F>;
                    }
                )*
                $(
                    impl<_F> $crate::r#struct::PiecewiseSimplify<$StructInitAs<$fdata, _F>> for $fact {
                        type Combined = $fact;
                        type Remainder = $StructInitAs<$fdata, _F>;
                    }
                )*
            )*
            $vis struct $StructSubstructData;
            $(
                $vis struct $sdata;
                $crate::struct_assoc_type!(impl $Struct::Fields::$s for $StructFieldData = $sdata);
                impl $crate::r#struct::SubstructOf<$Struct> for $sdata {
                    type Type = $sty;
                    fn get_field(
                        __field: $crate::arcmap::ArcMap<$StructContent>
                    ) -> $crate::arcmap::ArcMap<<$sty as $crate::r#struct::ShareableStruct>::Content> {
                        __field.map(|__field| &__field.$s)
                    }
                }
                impl<__Actions: $crate::r#struct::ActionsFor<$sty>> $crate::r#struct::InitSubstruct<$Struct, $sdata> for $StructInitAs<$sdata, __Actions> {
                    type Actions = __Actions;
                }
                impl<_F> $crate::r#struct::Simple for $StructInitAs<$sdata,_F> {}
                impl<_A, _F> $crate::r#struct::Append<_A> for $StructInitAs<$sdata,_F> {
                    type Appended = ($StructInitAs<$sdata,_F>, _A);
                }
                impl<_F> $crate::r#struct::PiecewiseSimplify<()> for $StructInitAs<$sdata,_F> {
                    type Combined = $StructInitAs<$sdata,_F>;
                    type Remainder = ();
                }
                impl<_F: $crate::r#struct::ActionsFor<$sty>, _G: $crate::r#struct::ActionsFor<$sty> + $crate::r#struct::LASimplify<_F>> $crate::r#struct::PiecewiseSimplify<$StructInitAs<$sdata,_F>> for $StructInitAs<$sdata,_G> {
                    type Combined = $StructInitAs<$sdata, _G::LASimplified>;
                    type Remainder = ();
                }
                $(
                    impl<_F: 'static> $crate::r#struct::InitSubstruct<$Struct, $others> for $StructInitAs<$sdata,_F> {
                        type Actions = ();
                    }
                    impl<_F: 'static, _G: 'static> $crate::r#struct::PiecewiseSimplify<$StructInitAs<$sdata, _F>> for $StructInitAs<$others, _G> {
                        type Combined = $StructInitAs<$others, _G>;
                        type Remainder = $StructInitAs<$sdata, _F>;
                    }
                )*
                $(
                    impl<_F: 'static> $crate::r#struct::InitField<$Struct, $field> for $StructInitAs<$sdata,_F> {
                        type Flag = ();
                    }
                    impl<_F, _G> $crate::r#struct::PiecewiseSimplify<$StructInitAs<$sdata, _F>> for $StructInitAs<$field, _G> {
                        type Combined = $StructInitAs<$field, _G>;
                        type Remainder = $StructInitAs<$sdata, _F>;
                    }
                )*
            )*
            $vis struct $StructActionData;
            $($crate::__alias_actions!(
                  vis: [$vis]
                  struct: $Struct
                  actiondata: $StructActionData
                  initas: $StructInitAs
                  name: [$($aty)?]
                  marker: [$($atymarker)?]
                  actions: [$abody]
                  fields: [$($afield)*]
                  substructs: [$($asubstruct)*]
                  other: [$($othera)*]
              );
            )*
            $vis struct $StructInitAs<__FieldMarker, __ActionOrFlag>(std::marker::PhantomData<(__FieldMarker, __ActionOrFlag)>);
            impl<__FieldMarker, __ActionOrFlag> Default for $StructInitAs<__FieldMarker, __ActionOrFlag> {
                fn default() -> Self {
                    Self(std::marker::PhantomData)
                }
            }
            impl<
                __Actions: $StructActions $(+ $crate::r#struct::InitField<$Struct,$fdata>)* $(+ $crate::r#struct::InitSubstruct<$Struct,$sdata>)*,
                __ImpliedActions: $StructActions $(+ $crate::r#struct::InitField<$Struct,$fdata>)* $(+ $crate::r#struct::InitSubstruct<$Struct,$sdata>)*,
            > $crate::r#struct::Implies<$StructInitAs<$Struct, __ImpliedActions>> for $StructInitAs<$Struct, __Actions>
            where
                $(<__Actions as $crate::r#struct::InitField<$Struct, $fdata>>::Flag: $crate::r#struct::Implies<<__ImpliedActions as $crate::r#struct::InitField<$Struct, $fdata>>::Flag>,)*
                $(<$sty as $crate::r#struct::ShareableStruct>::InitAs<$sty, <__Actions as $crate::r#struct::InitSubstruct<$Struct, $sdata>>::Actions>:
                    $crate::r#struct::Implies<<$sty as $crate::r#struct::ShareableStruct>::InitAs<$sty, <__ImpliedActions as $crate::r#struct::InitSubstruct<$Struct, $sdata>>::Actions>>,
                )*
            {
            }
            impl<__Actions: $StructActions, __ImpliedActions: $StructActions> std::convert::AsRef<$Struct<__ImpliedActions>> for $Struct<__Actions>
            where
                $StructInitAs<$Struct, __Actions>: $crate::r#struct::Implies<$StructInitAs<$Struct, __ImpliedActions>>
            {
                #[allow(clippy::transmute_ptr_to_ptr)]
                fn as_ref(&self) -> &$Struct<__ImpliedActions> {
                    // SAFETY:
                    //   * the layout of `Struct<A>` does not depend on `A`.
                    //   * the implementation of `$crate::r#strcut::Implies` guarantees that
                    //   nothing is treated as an initialized field when it has not been.
                    unsafe { std::mem::transmute(self) }
                }
            }
            impl $crate::r#struct::ShareableStruct for $Struct {
                type Content = $StructContent;
                type FieldData = $StructFieldData;
                type SubstructData = $StructSubstructData;
                type ActionData = $StructActionData;
                type InitAs<__FieldMarker, __ActionOrFlag> = $StructInitAs<__FieldMarker, __ActionOrFlag>;
            }
            impl<__Actions: Default $(+ $crate::r#struct::InitField<$Struct,$fdata>)* $(+ $crate::r#struct::InitSubstruct<$Struct,$sdata>)*>
                $StructActions for __Actions
            {
            }
            impl<__Actions: $StructActions> $crate::r#struct::HasActions<__Actions> for $Struct {
                type WithActions = $Struct<__Actions>;
                fn use_(listener: (usize, std::sync::Arc<dyn Send + Sync + Fn()>), content: $crate::arcmap::ArcMap<$StructContent>) -> $Struct<__Actions> {
                    $Struct {
                        $($f: <<__Actions as $crate::r#struct::InitField<$Struct, $fdata>>::Flag as $crate::r#struct::InitFlag>::init(listener.clone(), content.clone().map(|c| &c.$f)),)*
                        $($s: <<__Actions as $crate::r#struct::InitSubstruct<$Struct, $sdata>>::Actions as $crate::r#struct::ActionsFor<$sty>>::use_(listener.clone(), content.clone().map(|c| &c.$s)),)*
                        __actions: std::marker::PhantomData,
                    }
                }
            }
            impl<__Actions: $StructActions $(+ $crate::r#struct::InitField<$Struct,$fdata>)* $(+ $crate::r#struct::InitSubstruct<$Struct,$sdata>)*> $crate::r#struct::HasWriteActions<__Actions> for $Struct
            where
                $(<__Actions as $crate::r#struct::InitField<$Struct, $fdata>>::Flag: $crate::r#struct::ShareFlag,)*
                $(<__Actions as $crate::r#struct::InitSubstruct<$Struct, $sdata>>::Actions: $crate::r#struct::WriteActionsFor<$sty>,)*
            {
                fn share(content: $crate::arcmap::ArcMap<$StructContent>) -> $Struct<__Actions> {
                    $Struct {
                        $($f: <<__Actions as $crate::r#struct::InitField<$Struct, $fdata>>::Flag as $crate::r#struct::ShareFlag>::share(content.clone().map(|c| &c.$f)),)*
                        $($s: <<__Actions as $crate::r#struct::InitSubstruct<$Struct, $sdata>>::Actions as $crate::r#struct::WriteActionsFor<$sty>>::share(content.clone().map(|c| &c.$s)),)*
                        __actions: std::marker::PhantomData,
                    }
                }
            }
        };
    };
}

#[allow(clippy::module_name_repetitions)]
pub trait ShareableStruct: Sized {
    type Content: Content<For = Self>;
    type FieldData;
    type SubstructData;
    type ActionData;
    type InitAs<A, B>: Default;
}
pub trait Static: ShareableStruct {
    fn r#static(
    ) -> &'static std::sync::Mutex<Option<crate::arcmap::ArcMap<<Self as ShareableStruct>::Content>>>;
    fn get_static() -> crate::arcmap::ArcMap<Self::Content> {
        Self::r#static()
            .lock()
            .unwrap()
            .get_or_insert_with(Default::default)
            .clone()
    }
}

pub trait Content: 'static + Sized + Send + Sync + Default {
    type For: ShareableStruct<Content = Self>;
}
pub trait ShareableStructWithActions {
    type Base: ShareableStruct;
    type Actions: ActionsFor<Self::Base, WithActions = Self>;
}

pub trait HasActions<A>: ShareableStruct {
    type WithActions: ShareableStructWithActions<Base = Self, Actions = A>;
    fn use_(
        listener: (usize, std::sync::Arc<dyn Send + Sync + Fn()>),
        content: crate::arcmap::ArcMap<Self::Content>,
    ) -> Self::WithActions;
}
pub trait ActionsFor<S: ShareableStruct>: 'static + Default {
    type WithActions: ShareableStructWithActions<Base = S, Actions = Self>;
    fn use_(
        listener: (usize, std::sync::Arc<dyn Send + Sync + Fn()>),
        content: crate::arcmap::ArcMap<S::Content>,
    ) -> Self::WithActions;
}
impl<A: 'static + Default, S: HasActions<A>> ActionsFor<S> for A {
    type WithActions = S::WithActions;
    fn use_(
        listener: (usize, std::sync::Arc<dyn Send + Sync + Fn()>),
        content: crate::arcmap::ArcMap<S::Content>,
    ) -> Self::WithActions {
        S::use_(listener, content)
    }
}

pub trait HasWriteActions<A>: HasActions<A> {
    fn share(content: crate::arcmap::ArcMap<Self::Content>) -> Self::WithActions;
}
pub trait WriteActionsFor<S: ShareableStruct>: ActionsFor<S> {
    fn share(content: crate::arcmap::ArcMap<S::Content>) -> Self::WithActions;
}
impl<A: 'static + Default, S: HasWriteActions<A>> WriteActionsFor<S> for A {
    fn share(content: crate::arcmap::ArcMap<S::Content>) -> Self::WithActions {
        S::share(content)
    }
}
pub trait Implies<A> {}
impl<F: InitFlag, G: sealed::ImpliesFlag<F>> Implies<F> for G {}

pub trait FieldOf<S: ShareableStruct> {
    type Type: 'static + Send + Sync;
    fn get_field(
        f: crate::arcmap::ArcMap<S::Content>,
    ) -> crate::arcmap::ArcMap<crate::shared::Link<Self::Type>>;
}
pub trait SubstructOf<S: ShareableStruct> {
    type Type: ShareableStruct;
    fn get_field(
        f: crate::arcmap::ArcMap<S::Content>,
    ) -> crate::arcmap::ArcMap<<Self::Type as ShareableStruct>::Content>;
}

pub trait InitField<S: ShareableStruct, F: FieldOf<S>>: 'static {
    type Flag: InitFlag;
}
impl<S: ShareableStruct, F: FieldOf<S>> InitField<S, F> for () {
    type Flag = ();
}
impl<S: ShareableStruct, F: FieldOf<S>, A: InitField<S, F>, B: InitField<S, F>> InitField<S, F>
    for (A, B)
where
    A::Flag: CombineFlag<B::Flag>,
{
    type Flag = <A::Flag as sealed::CombineFlag<B::Flag>>::Combined;
}

pub trait InitSubstruct<S: ShareableStruct, U: SubstructOf<S>>: 'static {
    type Actions: ActionsFor<U::Type>;
}
impl<S: ShareableStruct, U: SubstructOf<S>> InitSubstruct<S, U> for ()
where
    (): ActionsFor<U::Type>,
{
    type Actions = ();
}
impl<S: ShareableStruct, U: SubstructOf<S>, A: InitSubstruct<S, U>, B: InitSubstruct<S, U>>
    InitSubstruct<S, U> for (A, B)
where
    (A::Actions, B::Actions): Simplify,
    <(A::Actions, B::Actions) as Simplify>::Simplified: ActionsFor<U::Type>,
{
    type Actions = <(A::Actions, B::Actions) as Simplify>::Simplified; // TODO: Simplify this down!
}

mod sealed {
    pub trait InitFlag: 'static + Sized {
        fn _init<T: 'static + Send + Sync>(
            listener: (usize, std::sync::Arc<dyn Send + Sync + Fn()>),
            link: crate::arcmap::ArcMap<crate::shared::Link<T>>,
        ) -> Option<crate::shared::Shared<T, Self>>;
    }
    impl InitFlag for () {
        fn _init<T: 'static + Send + Sync>(
            _listener: (usize, std::sync::Arc<dyn Send + Sync + Fn()>),
            _link: crate::arcmap::ArcMap<crate::shared::Link<T>>,
        ) -> Option<crate::shared::Shared<T, Self>> {
            None
        }
    }
    impl InitFlag for crate::W {
        fn _init<T: 'static + Send + Sync>(
            listener: (usize, std::sync::Arc<dyn Send + Sync + Fn()>),
            link: crate::arcmap::ArcMap<crate::shared::Link<T>>,
        ) -> Option<crate::shared::Shared<T, Self>> {
            let mut shareable = crate::shared::Shareable(Some(link));
            Some(crate::shared::Shared::init_with_listener(
                listener,
                &mut shareable,
                || unreachable!(),
            ))
        }
    }
    impl InitFlag for crate::RW {
        fn _init<T: 'static + Send + Sync>(
            listener: (usize, std::sync::Arc<dyn Send + Sync + Fn()>),
            link: crate::arcmap::ArcMap<crate::shared::Link<T>>,
        ) -> Option<crate::shared::Shared<T, Self>> {
            let mut shareable = crate::shared::Shareable(Some(link));
            Some(crate::shared::Shared::init_with_listener(
                listener,
                &mut shareable,
                || unreachable!(),
            ))
        }
    }
    pub trait ShareFlag: Sized {
        fn _share<T: 'static + Send + Sync>(
            link: crate::arcmap::ArcMap<crate::shared::Link<T>>,
        ) -> Option<crate::shared::Shared<T, Self>>;
    }
    impl ShareFlag for () {
        fn _share<T: 'static + Send + Sync>(
            _link: crate::arcmap::ArcMap<crate::shared::Link<T>>,
        ) -> Option<crate::shared::Shared<T, Self>> {
            None
        }
    }
    impl ShareFlag for crate::W {
        fn _share<T: 'static + Send + Sync>(
            link: crate::arcmap::ArcMap<crate::shared::Link<T>>,
        ) -> Option<crate::shared::Shared<T, Self>> {
            Some(crate::shared::Static::_share(link))
        }
    }
    pub trait CombineFlag<Rhs: InitFlag>: InitFlag {
        type Combined: InitFlag;
    }
    impl CombineFlag<()> for () {
        type Combined = ();
    }
    impl CombineFlag<crate::W> for () {
        type Combined = crate::W;
    }
    impl CombineFlag<crate::RW> for () {
        type Combined = crate::RW;
    }
    impl CombineFlag<()> for crate::W {
        type Combined = crate::W;
    }
    impl CombineFlag<crate::W> for crate::W {
        type Combined = crate::W;
    }
    impl CombineFlag<crate::RW> for crate::W {
        type Combined = crate::RW;
    }
    impl CombineFlag<()> for crate::RW {
        type Combined = crate::RW;
    }
    impl CombineFlag<crate::W> for crate::RW {
        type Combined = crate::RW;
    }
    impl CombineFlag<crate::RW> for crate::RW {
        type Combined = crate::RW;
    }

    pub trait ImpliesFlag<F: InitFlag>: InitFlag {}
    impl<F: InitFlag> ImpliesFlag<()> for F {}
    impl ImpliesFlag<crate::W> for crate::W {}
    impl ImpliesFlag<crate::W> for crate::RW {}
    impl ImpliesFlag<crate::RW> for crate::RW {}
}
pub trait InitFlag: sealed::InitFlag {
    fn init<T: 'static + Send + Sync>(
        listener: (usize, std::sync::Arc<dyn Send + Sync + Fn()>),
        link: crate::arcmap::ArcMap<crate::shared::Link<T>>,
    ) -> Option<crate::shared::Shared<T, Self>>;
}
impl<F: sealed::InitFlag> InitFlag for F {
    fn init<T: 'static + Send + Sync>(
        listener: (usize, std::sync::Arc<dyn Send + Sync + Fn()>),
        link: crate::arcmap::ArcMap<crate::shared::Link<T>>,
    ) -> Option<crate::shared::Shared<T, Self>> {
        F::_init(listener, link)
    }
}
pub trait ShareFlag: sealed::ShareFlag {
    fn share<T: 'static + Send + Sync>(
        link: crate::arcmap::ArcMap<crate::shared::Link<T>>,
    ) -> Option<crate::shared::Shared<T, Self>>;
}
impl<F: sealed::ShareFlag> ShareFlag for F {
    fn share<T: 'static + Send + Sync>(
        link: crate::arcmap::ArcMap<crate::shared::Link<T>>,
    ) -> Option<crate::shared::Shared<T, Self>> {
        F::_share(link)
    }
}

pub trait CombineFlag<F: InitFlag>: sealed::CombineFlag<F> {}
impl<F: InitFlag, G: sealed::CombineFlag<F>> CombineFlag<F> for G {}

pub trait Simple {}
impl Simple for () {}

pub trait LASimplify<A> {
    type LASimplified;
}
impl<A: Simple, B> LASimplify<B> for A
where
    (A, B): RASimplify,
{
    type LASimplified = <(A, B) as RASimplify>::RASimplified;
}
impl<A: Simple, C, B: LASimplify<C>> LASimplify<C> for (A, B)
where
    (A, B::LASimplified): RASimplify,
{
    type LASimplified = <(A, B::LASimplified) as RASimplify>::RASimplified;
}

pub trait PiecewiseSimplify<A: Simple> {
    type Combined;
    type Remainder: Simple;
}
impl<A: Simple, B, C: Simple> PiecewiseSimplify<C> for (A, B)
where
    A: PiecewiseSimplify<C>,
    B: PiecewiseSimplify<A::Remainder>,
{
    type Combined = (
        A::Combined,
        <B as PiecewiseSimplify<A::Remainder>>::Combined,
    );
    type Remainder = <B as PiecewiseSimplify<A::Remainder>>::Remainder;
}
impl<A: Simple> PiecewiseSimplify<A> for () {
    type Combined = A;
    type Remainder = ();
}
pub trait Append<A> {
    type Appended;
}
impl<A> Append<A> for () {
    type Appended = A;
}

pub trait Simplify {
    type Simplified;
}
impl<A, B> Simplify for (A, B)
where
    A: LASimplify<B>,
{
    type Simplified = A::LASimplified;
}

pub trait RASimplify {
    type RASimplified;
}
impl<A: Simple> RASimplify for A {
    type RASimplified = A;
}
impl<A: Simple, B: PiecewiseSimplify<A>> RASimplify for (A, B)
where
    B::Remainder: Append<B::Combined>,
{
    type RASimplified = <B::Remainder as Append<B::Combined>>::Appended;
}

#[macro_export]
#[allow(clippy::module_name_repetitions)]
macro_rules! struct_actions {
    ($Struct:ty { $field:ident: $flag:ident$(, $($($bod:tt)+)?)? }) => {
        (<$Struct as $crate::r#struct::ShareableStruct>::InitAs<$crate::r#struct_assoc_type!({$Struct}::Fields::$field), $crate::$flag>$($(, $crate::struct_actions!($Struct { $($bod)* }))?)?)
    };
    ($Struct:ty { |$field:ident: {$($flags:tt)*}$(, $($($bod:tt)+)?)? }) => {
        (<$Struct as $crate::r#struct::ShareableStruct>::InitAs<
            $crate::r#struct_assoc_type!({$Struct}::Fields::$field),
            $crate::struct_actions!(<$crate::struct_assoc_type!({$Struct}::Fields::$field) as $crate::r#struct::SubstructOf<$Struct>>::Type {$($flags)*})
        >$($(, $crate::struct_actions!($Struct { $($bod)* }))?)?)
    };
    ($Struct:ty { $A:ident$(, $($($bod:tt)+)?)? }) => {
        ($crate::struct_assoc_type!({$Struct}::Actions::$A)$($(, $crate::struct_actions!($Struct { $($bod)* }))?)?)
    };
    ($Struct:ty {}) => {()};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __alias_actions {
    ( vis: [$vis:vis]
      struct: $Struct:ident
      actiondata: $StructActionData:ident
      initas: $StructInitAs:ident
      name: []
      $($r:tt)*
    ) => {};
    ( vis: [$vis:vis]
      struct: $Struct:ident
      actiondata: $StructActionData:ident
      initas: $StructInitAs:ident
      name: [$a:ident]
      marker: [$m:ident]
      actions: [$ty:ty]
      fields: [$($field:ident)*]
      substructs: [$($substruct:ident)*]
      other: [$($othera:ident)*]
    ) => {
        #[derive(Default)]
        $vis struct $m;
        $crate::struct_assoc_type!(impl $Struct::Actions::$a for $StructActionData = $m);
        $(impl $crate::r#struct::InitField<$Struct, $field> for $a {
            type Flag = <$ty as $crate::r#struct::InitField<$Struct, $field>>::Flag;
        })*
        $(impl $crate::r#struct::InitSubstruct<$Struct, $substruct> for $a {
            type Actions = <$ty as $crate::r#struct::InitSubstruct<$Struct, $substruct>>::Actions;
        })*
        impl $crate::r#struct::Simple for $m {}
        impl<_A> $crate::r#struct::Append<_A> for $m {
            type Appended = ($m, _A);
        }
        impl $crate::r#struct::PiecewiseSimplify<()> for $m {
            type Combined = $m;
            type Remainder = ();
        }
        impl $crate::r#struct::PiecewiseSimplify<$m> for $m {
            type Combined = $m;
            type Remainder = ();
        }
        impl<__FieldMarker, __ActionOrFlag> $crate::r#struct::PiecewiseSimplify<$m> for $StructInitAs<__FieldMarker, __ActionOrFlag> {
            type Combined = $StructInitAs<__FieldMarker, __ActionOrFlag>;
            type Remainder = $m;
        }
    };
}
