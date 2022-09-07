# Dioxus Shareables

`dioxus-shareables` adds hooks for sharing structures between `dioxus` components. Version 1.0.x provides three interfaces:

1. The `shareable!()` macro creates a single shared value.

    ```rust
    use dioxus::prelude::*;
    use dioxus_shareables::shareable;

    shareable!(Var: usize = 900);

    #[allow(non_snake_case)]
    pub fn Reader(cx: Scope) -> Element {
        let r = *Var.use_rw(&cx).read(); // this component will update when Var changes.
        // ...
    }

    #[allow(non_snake_case)]
    pub fn Writer(cx: Scope) -> Element {
        let w1 = Var.use_w(&cx); // this component writes to Var, but does not get updated when Var
                                 // changes
        // ...
    }
    ```
1. A `List` provides an array of shared values.
    Using a `List<T>` rather than a `Vec<T>` allows components which use only one or two list items to get updated only when the specific list items they use are changed.

    ```rust
    use dioxus::prelude::*;
    use dioxus_shareables::{shareable, List, ListEntry};
    
    shareable!(Numbers: List<usize> = [3, 5, 7].into_iter().collect());
    
    #[allow(non_snake_case)]
    fn IterateOverNumbers(cx: Scope) -> Element {
        let nums = Numbers.use_rw(&cx); // This component is updated when new items are added to or
                                        // removed from the list, but not when the individual list
                                        // items change.
        let w = nums.clone();
        cx.render(rsx! {
            ul {
                nums.read().iter().map(|n| rsx! { ReadANumber { num: n } })
            }
        })
    }
    
    #[allow(non_snake_case)]
    #[inline_props]
    fn ReadANumber(cx: Scope, num: ListEntry<usize>) -> Element {
        let num = num.use_rw(&cx); // This component is updated when this specific entry in the
                                   // list is modified, but not when the others are.
        ...
    }
    ```
    `List` is a `Vec` internally, and the methods it implements therefore get their names and behavior from `Vec`.
1. The `shareable_struct!{}` macro provides a shared `struct` with interfaces that encapsulate different behavior.
    The idea is that each field of the struct will be stored in a separate global, and loaded only when requested. The actions block describes possible ways of using the struct in terms of what type of access (`W` or `RW`) they need to fields of the struct.
    The struct can then be initialized using an "action" which describes which fields we need which type of access to.

    ```rust
    use dioxus::prelude::*;
    dioxus_shareables::shareable_struct! {
        pub struct Fuzzy {
            wuzzy: u8 = 17,
            was_a: u16 = 59,
            was_he: &'static str = "bear?",
        }
        actions for Puzzle {
           pub WAS: pub WasTrait = W[was_a, was_he]; // declares an WAS constant, as well an
                                                     // equivalent trait.
           INIT = W[wuzzy, was_a], RW[was_he]; // declares the INIT constant, but no
                                               // equivalent trait.
        }
    };
    impl<A: FuzzyActions> Fuzzy<A> {
        pub fn method(&self) where A: WasTrait {
           let me = self.with_actions(WAS); // Pending updates to the rust trait system, we
                                            // have to typecast here to get a Fuzzy<WAS>.
           *me.was_he().write() = "bare!"; // We have write access to was_he
           // self.wuzzy(); // but this would fail because we don't have access to wuzzy.
           // ...
        }
    }
    // ...
    fn component(cx: Scope) -> Element {
         let fuzzy = Fuzzy::use_(&cx, INIT); // This creates the hooks for the struct and initializes it
                                             // from the necessary globals.
         // ...
         fuzzy.method(); // This is ok, since the INIT action includes everything the WAS action does.
         // ...
    }
    ```
